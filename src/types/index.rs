use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;
use dto::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, TagActiveModel, TagEntity, ZettelEntity, ZettelModelEx, ZettelTagActiveModel,
    ZettelTagColumns, ZettelTagEntity,
};
use rayon::iter::{ParallelBridge, ParallelIterator};
use tracing::info;

use crate::types::{FrontMatter, ZettelId, frontmatter::Body};

#[derive(Debug, Clone)]
pub struct Index {
    pub(super) zods: HashMap<ZettelId, ZettelOnDisk>,
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
            .map(|entry| -> Result<(ZettelId, ZettelOnDisk)> {
                let path = entry.path();
                let id: ZettelId = path.as_path().try_into()?;
                let (fm, body) = FrontMatter::extract_from_file(&path)?;

                Ok((
                    id,
                    ZettelOnDisk {
                        fm,
                        body,
                        path: path.canonicalize()?,
                    },
                ))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            // .par_bridge()
            .for_each(|(id, zod)| {
                zods.insert(id, zod);
            });

        Ok(Self { zods })
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
    pub async fn sync_title_with_db(
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

    pub fn get_zod(&self, zid: &ZettelId) -> &ZettelOnDisk {
        self.zods.get(zid).expect("Invariant broken. Any zid we lookup must exist in the index, otherwise the db is corrupt or not sync'd.")
    }

    fn get_zod_mut(&mut self, zid: &ZettelId) -> &mut ZettelOnDisk {
        self.zods.get_mut(zid).expect("Invariant broken. Any zid we lookup must exist in the index, otherwise the db is corrupt or not sync'd.")
    }

    pub const fn zods(&self) -> &HashMap<ZettelId, ZettelOnDisk> {
        &self.zods
    }

    pub fn sync_with_db(&self, _db: &DatabaseConnection) {
        todo!()
    }

    //NOTE: we dont support links just yet
    // fn parse_links(src: &ZettelId, body: Body) -> Result<Vec<Link>> {
    //     let parsed = Parser::new(&body);

    //     let mut links = vec![];

    //     for event in parsed {
    //         if let Event::Start(MkTag::Link { dest_url, .. }) = event {
    //             info!("Found dest_url: {dest_url:#?}");

    //             let dest_path = {
    //                 // remove leading "./"
    //                 let without_prefix = dest_url.strip_prefix("./").unwrap_or(&dest_url);

    //                 // remove "#" and everything after it
    //                 let without_anchor = without_prefix.split('#').next().unwrap();

    //                 // add .md if not present
    //                 let normalized = if std::path::Path::new(without_anchor)
    //                     .extension()
    //                     .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
    //                 {
    //                     without_anchor.to_string()
    //                 } else {
    //                     format!("{without_anchor}.md")
    //                 };

    //                 let mut tmp_root = self
    //                     .zods
    //                     .get(src)
    //                     .expect("Invariant Broken! src must exist inside index")
    //                     .path
    //                     .clone();
    //                 tmp_root.push(normalized);
    //                 tmp_root
    //             };
    //             // simplest way to validate that the path exists
    //             let Ok(canon_url) = dest_path.canonicalize() else {
    //                 error!("Link not found!: {dest_path:?}");
    //                 continue;
    //             };

    //             // TODO: check that the thing actually exists inside the ws.db
    //             // instead of just seeing if we can turn it into a ZettelId
    //             let dst_id = ZettelId::try_from(canon_url)?;

    //             let link = Link::new(src.clone(), dst_id);

    //             links.push(link);
    //         }
    //     }

    //     Ok(links)
    // }
}
