use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;
use dto::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    NanoId, QueryFilter, TagActiveModel, TagEntity, ZettelEntity, ZettelModelEx,
    ZettelTagActiveModel, ZettelTagColumns, ZettelTagEntity,
};
use pulldown_cmark::{Event, Options, Parser};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tracing::info;

use crate::types::{FrontMatter, Link, ZettelId, frontmatter::Body};

#[derive(Debug, Clone)]
pub struct Index {
    pub(super) zods: HashMap<ZettelId, ZettelOnDisk>,
    pub outgoing_links: HashMap<ZettelId, Vec<Link>>,
    // pub(super) incoming_links: HashMap<ZettelId, Vec<Link>>,
}

#[derive(Debug, Clone)]
pub struct ZettelOnDisk {
    pub fm: FrontMatter,
    pub body: Body,
    pub path: PathBuf,
}

impl Index {
    /// Parses the `root` path to construct an `Index`.
    pub fn tabulate(root: &Path) -> Result<Self> {
        let root = root.canonicalize()?;

        let mut zods = HashMap::new();
        let mut possible_outgoing_links = HashMap::new();

        std::fs::read_dir(root)?
            .par_bridge()
            .flatten()
            .filter(|entry| {
                entry.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                    && entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .is_some_and(|ext| ext == "md")
            })
            .map(|entry| -> Result<(ZettelId, ZettelOnDisk, Vec<Link>)> {
                let path = entry.path();
                let zid: ZettelId = path.as_path().try_into()?;
                let (fm, body) = FrontMatter::extract_from_file(&path)?;

                let outgoing_links = Self::_parse_outgoing_links(&zid, &body);

                Ok((
                    zid,
                    ZettelOnDisk {
                        fm,
                        body,
                        path: path.canonicalize()?,
                    },
                    outgoing_links,
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            // .par_bridge()
            .for_each(|(id, zod, zettel_outgoing_links)| {
                zods.insert(id.clone(), zod);
                possible_outgoing_links.insert(id, zettel_outgoing_links);
            });

        // simple validation step for links
        let zid_set = zods.keys().cloned().collect::<HashSet<_>>();

        let outgoing_links = possible_outgoing_links
            .into_iter()
            .map(|(id, links)| {
                let valid_links = links
                    .into_iter()
                    .filter(|link| zid_set.contains(&link.source) && zid_set.contains(&link.dest))
                    .collect::<Vec<_>>();

                (id, valid_links)
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            zods,
            outgoing_links,
        })
    }

    pub fn parse_outgoing_links(&mut self, zid: &ZettelId) {
        let body = self.get_zod(zid).body.as_str();
        let links = Self::_parse_outgoing_links(zid, body);
        self.outgoing_links.insert(zid.clone(), links);
    }

    fn _parse_outgoing_links(zid: &ZettelId, body: &str) -> Vec<Link> {
        let parser = Parser::new_ext(body, Options::ENABLE_WIKILINKS);

        // let mut links = vec![];
        let mut links = vec![];

        for event in parser {
            if let Event::Start(pulldown_cmark::Tag::Link {
                link_type: pulldown_cmark::LinkType::WikiLink { has_pothole: _ },
                dest_url,
                ..
            }) = event
            {
                let nano_id = dest_url.as_ref();

                // how do we validate that this is a proper link?
                // we are just going to trust them for now

                links.push(Link::new(zid.clone(), NanoId::from(nano_id)));
            }
        }

        links
    }

    pub fn update_path_for_zid(&mut self, zid: &ZettelId, new_path: PathBuf) {
        self.get_zod_mut(zid).path = new_path;
    }

    /// Updates the interal state of the `Index` for the provided `Zid`.
    pub fn process_zid(&mut self, zid: &ZettelId) -> Result<()> {
        let zod = self.get_zod_mut(zid);

        let (fm, body) = FrontMatter::extract_from_file(&zod.path)?;

        zod.fm = fm;
        zod.body = body;

        Ok(())
    }

    /// Sync's the curren title of the `Zettel` with the
    /// provided `zid` with the `DB`
    pub async fn sync_zettel_title_with_db(
        &mut self,
        zid: &ZettelId,
        db: &DatabaseConnection,
    ) -> Result<()> {
        let fm = &mut self.get_zod_mut(zid).fm;

        let mut model = ZettelEntity::find_by_nano_id(zid.clone())
            .one(db)
            .await?
            .expect("this must exist")
            .into_active_model();

        model.title = ActiveValue::Set(fm.title.clone());

        model.update(db).await?;

        info!("We updated the zettel: {zid:#?}");

        Ok(())
    }

    /// Sync's `Tag`'s that are present in the frontmatter of this
    /// `Zettel` to the database.
    pub async fn sync_tags_with_db(
        &mut self,
        zid: &ZettelId,
        db: &DatabaseConnection,
    ) -> Result<()> {
        let fm = &mut self.get_zod_mut(zid).fm;

        let mut tag_strings = fm.tag_strings.clone();

        tag_strings.sort();

        let db_zettel: ZettelModelEx = ZettelEntity::load()
            .with(TagEntity)
            .filter_by_nano_id(zid.clone())
            .one(db)
            .await?
            .expect("Invariant broken, zettel should not be deleted");

        for db_tag in db_zettel.tags {
            if let Ok(idx) = tag_strings.binary_search(&db_tag.name) {
                // we remove tags we have already processed
                tag_strings.remove(idx);
            } else {
                // the db says the file has tag `x`, but that tag is missing from the
                // front matter, we can assume its gone, lets delete that link
                let to_remove = ZettelTagEntity::find()
                    .filter(ZettelTagColumns::ZettelNanoId.eq(zid.to_string()))
                    .filter(ZettelTagColumns::TagNanoId.eq(db_tag.nano_id))
                    .one(db)
                    .await?
                    .expect("this link must exist");

                to_remove.into_active_model().delete(db).await?;
            }
        }

        // now any tags that are left inside zettel_tag_strings,
        // we have to look up the tags in the db and then reset them?
        for tag_str in tag_strings {
            // this is the tag that either already exists with this name, or we just created this new one
            let tag =
                if let Some(existing) = TagEntity::load().filter_by_name(&tag_str).one(db).await? {
                    existing
                } else {
                    let am = TagActiveModel {
                        name: ActiveValue::Set(tag_str.clone()),
                        ..Default::default()
                    };

                    am.insert(db).await?.into()
                };

            // this zettel has this tag now
            let _ = ZettelTagActiveModel {
                zettel_nano_id: ActiveValue::Set(zid.to_string()),
                tag_nano_id: ActiveValue::Set(tag.nano_id.to_string()),
            }
            .insert(db)
            .await?;
        }
        Ok(())
    }

    pub fn get_links(&self, zid: &ZettelId) -> &Vec<Link> {
        self.outgoing_links
            .get(zid)
            .expect("Invariant broken. Any zid we look up exist inside this map")
    }

    pub fn sync_with_db(&self, _db: &DatabaseConnection) {
        todo!()
    }

    pub fn get_zod(&self, zid: &ZettelId) -> &ZettelOnDisk {
        self.zods.get(zid).expect("Invariant broken. Any zid we lookup must exist in the index, otherwise the db is corrupt or not sync'd.")
    }

    pub fn get_zod_mut(&mut self, zid: &ZettelId) -> &mut ZettelOnDisk {
        self.zods.get_mut(zid).expect("Invariant broken. Any zid we lookup must exist in the index, otherwise the db is corrupt or not sync'd.")
    }

    pub const fn zods(&self) -> &HashMap<ZettelId, ZettelOnDisk> {
        &self.zods
    }
}
