use color_eyre::eyre::Result;
use dto::{
    DateTime, GroupActiveModel, GroupEntity, GroupModelEx, IntoActiveModel as _, NanoId,
    TagActiveModel, TagEntity, ZettelEntity,
};
use tree::Node;

use crate::types::{Kasten, Priority, Tag, TodoNode, Zettel, frontmatter};

/// A `Group` which contains tasks!
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Group {
    /// Should only be constructed from models.
    _private: (),

    pub id: NanoId,
    pub name: String,
    pub priority: Priority,
    pub parent_id: Option<NanoId>,
    pub created_at: DateTime,
    pub modified_at: DateTime,
    /// The `Zettel` that is related to this `Group`.
    /// Can store notes regarding this group in
    /// the `Zettel`
    pub zettel: Zettel,

    /// The `Tag` that is related to this `Group`
    pub tag: Tag,
}

impl Group {
    pub fn created_at(&self) -> String {
        self.created_at
            .format(frontmatter::DATE_FMT_STR)
            .to_string()
    }
    pub fn modified_at(&self) -> String {
        self.modified_at
            .format(frontmatter::DATE_FMT_STR)
            .to_string()
    }

    pub async fn new(
        name: impl Into<String>,
        parent_id: Option<NanoId>,
        kt: &mut Kasten,
    ) -> Result<Self> {
        let name = name.into();
        let tag: Tag = TagActiveModel::builder()
            .set_name(name.clone())
            .insert(&kt.db)
            .await?
            .into();

        let tag_id = tag.id.clone();

        // then create the zettel for the group
        let zettel = Zettel::new(name.clone(), kt, vec![tag]).await?;

        // then insert that shi
        let inserted = GroupActiveModel::builder()
            .set_name(name)
            .set_parent_group_id(parent_id)
            .set_tag(
                TagEntity::load()
                    .filter_by_nano_id(tag_id)
                    .one(&kt.db)
                    .await?
                    .expect("Tag must exist since we just created it")
                    .into_active_model(),
            )
            .set_zettel(
                ZettelEntity::load()
                    .filter_by_nano_id(zettel.id)
                    .one(&kt.db)
                    .await?
                    .expect("Zettel must exist since we just created it")
                    .into_active_model(),
            )
            .set_priority(Priority::default())
            .insert(&kt.db)
            .await?;

        // group should also have the accompanying tag for it.
        let group: Self = GroupEntity::load()
            .with(TagEntity)
            .with((ZettelEntity, TagEntity))
            .filter_by_nano_id(inserted.nano_id)
            .one(&kt.db)
            .await?
            .expect("We just inserted it")
            .into();

        // we should also insert the group into the kasten

        let parent_node_id = group
            .parent_id
            .clone()
            .and_then(|id| kt.todo_tree.nanoid_to_nodeid.get(&id))
            .unwrap_or(&kt.todo_tree.root_id);

        let my_depth = if *parent_node_id == kt.todo_tree.root_id {
            0
        } else {
            kt.todo_tree
                .tree
                .get(parent_node_id)
                .expect("Must exist inside tree")
                .data()
                .depth
                + 1
        };

        let inserted_node_id = kt
            .todo_tree
            .tree
            .insert(
                Node::new(TodoNode::new(
                    super::TodoNodeKind::Group(Box::new(group.clone())),
                    my_depth,
                )),
                tree::InsertBehavior::UnderNode(parent_node_id),
            )
            .expect("Insertion of group should not error!");

        kt.todo_tree
            .nanoid_to_nodeid
            .insert(group.id.clone(), inserted_node_id);

        Ok(group)
    }
}

impl From<GroupModelEx> for Group {
    fn from(value: GroupModelEx) -> Self {
        Self {
            _private: (),
            id: value.nano_id,
            name: value.name,
            priority: value.priority.into(),
            parent_id: value.parent_group_id,
            created_at: value.created_at,
            modified_at: value.modified_at,
            zettel: value
                .zettel
                .into_option()
                .expect(
                    "When fetching a Group from the database, we expect to always have the Zettel loaded!!",
                )
                .into(),
            tag: value
            .tag
            .into_option()
            .expect(
                "When fetching a Group from the database, we expect to always have the Tag loaded!!",
            )
            .into(),
        }
    }
}
