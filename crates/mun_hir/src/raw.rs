use mun_syntax::ast::{self, ModuleItemOwner, NameOwner};

use crate::{
    arena::{Arena, Idx},
    name::AsName,
    DefDatabase, FileAstId, FileId, Name,
};
use std::ops::Index;
use std::sync::Arc;

/// `RawItems` are top level file items. `RawItems` do not change on most edits.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RawItems {
    definitions: Arena<DefData>,
    items: Vec<RawFileItem>,
}

/// Represents an id of a `DefData` in `RawItems`.
pub(super) type LocalDefId = Idx<DefData>;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct DefData {
    pub(super) name: Name,
    pub(super) kind: DefKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum DefKind {
    Function(FileAstId<ast::FunctionDef>),
    Struct(FileAstId<ast::StructDef>),
    TypeAlias(FileAstId<ast::TypeAliasDef>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum RawFileItem {
    Definition(LocalDefId),
}

impl Index<LocalDefId> for RawItems {
    type Output = DefData;

    fn index(&self, index: LocalDefId) -> &Self::Output {
        &self.definitions[index]
    }
}

impl RawItems {
    pub(crate) fn raw_file_items_query(db: &dyn DefDatabase, file_id: FileId) -> Arc<RawItems> {
        let mut items = RawItems::default();

        let source_file = db.parse(file_id).tree();
        let ast_id_map = db.ast_id_map(file_id);

        // Iterate over all items in the source file
        for item in source_file.items() {
            let (kind, name) = match item.kind() {
                ast::ModuleItemKind::FunctionDef(it) => {
                    (DefKind::Function((*ast_id_map).ast_id(&it)), it.name())
                }
                ast::ModuleItemKind::StructDef(it) => {
                    (DefKind::Struct((*ast_id_map).ast_id(&it)), it.name())
                }
                ast::ModuleItemKind::TypeAliasDef(it) => {
                    (DefKind::TypeAlias((*ast_id_map).ast_id(&it)), it.name())
                }
            };

            // If no name is provided an error is already emitted
            if let Some(name) = name {
                let id = items.definitions.alloc(DefData {
                    name: name.as_name(),
                    kind,
                });
                items.items.push(RawFileItem::Definition(id));
            }
        }
        Arc::new(items)
    }

    pub(super) fn items(&self) -> &[RawFileItem] {
        &self.items
    }
}
