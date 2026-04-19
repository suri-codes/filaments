use color_eyre::eyre::Result;
use dto::{IntoActiveModel as _, NanoId, TagEntity, TagModel, TagModelEx, ZettelEntity};

use crate::types::{Color, Kasten, Zettel};

/// Represents a `Tag` in a `ZettelKasten` note taking method.
/// Easy way to link multiple notes under one simple word.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tag {
    /// Should only be constructed from models.
    _private: (),

    /// A unique `NanoId`
    pub id: NanoId,
    /// Name of the tag
    pub name: String,
    /// Color of the tag
    pub color: Color,
}

impl Tag {
    pub async fn alter_name(
        id: NanoId,
        new_name: impl Into<String>,
        kt: &mut Kasten,
    ) -> Result<()> {
        let new_name = new_name.into();

        TagEntity::load()
            .filter_by_nano_id(id.clone())
            .one(&kt.db)
            .await?
            .expect("Invariant Broken: Must exist")
            .into_active_model()
            .set_name(new_name.as_str())
            .update(&kt.db)
            .await?;

        // fetch all the zettels for this tag
        let tag = TagEntity::load()
            .filter_by_nano_id(id)
            .with(ZettelEntity)
            .one(&kt.db)
            .await?
            .expect("We just saved it");

        assert!(
            tag.zettels.is_loaded(),
            "We expect the zettels to be loaded"
        );

        for zettel in tag.zettels {
            Zettel::write_tags_from_db(zettel.nano_id.into(), kt).await?;
        }

        Ok(())
    }
}

impl From<TagModel> for Tag {
    fn from(value: TagModel) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            color: value.color.into(),
        }
    }
}

impl From<TagModelEx> for Tag {
    fn from(value: TagModelEx) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            color: value.color.into(),
        }
    }
}
