use std::usize;

use super::{Expr, InstanceScheme, PathQuery, Scheme, TypeID};
use crate::com::loc::Loc;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AnyID {
    Variable(VariableID),
    UserType(UserTypeID),
    Record(RecordID),
    Union(UnionID),
    Class(ClassID),
    Instance(InstanceID),
    Import(ImportID),
    Alias(AliasID),
}

define_id_type!(VariableID, Variable);
define_id_type!(UserTypeID, UserType);
define_id_type!(RecordID, Record);
define_id_type!(UnionID, Union);
define_id_type!(ClassID, Class);
define_id_type!(InstanceID, Instance);
define_id_type!(ImportID, Import);
define_id_type!(AliasID, Alias);

impl VariableID {
    pub fn dummy() -> Self {
        Self(usize::MAX)
    }
}

#[derive(Default)]
pub struct Entities {
    pub variables: Vec<VariableInfo>,
    pub user_types: Vec<UserTypeInfo>,
    pub records: Vec<RecordInfo>,
    pub unions: Vec<UnionInfo>,
    pub classes: Vec<ClassInfo>,
    pub instances: Vec<InstanceInfo>,
    pub imports: Vec<ImportInfo>,
    pub aliases: Vec<AliasInfo>,
}

impl Entities {
    entity_impl!(
        variables,
        (create_variable, next_variable_id, get_variable_info get_variable_info_mut),
        (VariableID => VariableInfo),
    );
    entity_impl!(
        user_types,
        (create_user_type, next_user_type_id, get_user_type_info get_user_type_info_mut),
        (UserTypeID => UserTypeInfo),
    );
    entity_impl!(
        records,
        (create_record, next_record_id, get_record_info get_record_info_mut),
        (RecordID => RecordInfo),
    );
    entity_impl!(
        unions,
        (create_union, next_union_id, get_union_info get_union_info_mut),
        (UnionID => UnionInfo),
    );
    entity_impl!(
        instances,
        (create_instance, next_instance_id, get_instance_info get_instance_info_mut),
        (InstanceID => InstanceInfo),
    );
    entity_impl!(
        classes,
        (create_class, next_class_id, get_class_info get_class_info_mut),
        (ClassID => ClassInfo),
    );
    entity_impl!(
        imports,
        (create_import, next_import_id, get_import_info get_imports_info_mut),
        (ImportID => ImportInfo),
    );
    entity_impl!(
        aliases,
        (create_alias, next_alias_id, get_alias_info get_alias_info_mut),
        (AliasID => AliasInfo),
    );

    pub fn get_record_field_info(
        &self,
        id: RecordID,
        tag: usize,
    ) -> (&RecordInfo, &RecordFieldInfo) {
        let info = self.get_record_info(id);
        (info, &info.fields[tag])
    }

    pub fn get_union_variant_info(&self, id: UnionID, tag: usize) -> (&UnionInfo, &VariantInfo) {
        let info = self.get_union_info(id);
        (info, &info.variants[tag])
    }

    pub fn get_class_item_info(&self, id: ClassID, index: usize) -> &ClassItemInfo {
        let info = self.get_class_info(id);
        &info.items[index]
    }
}

pub struct VariableInfo {
    pub name: String,
    pub scheme: Scheme,
    pub loc: Loc,
    pub depth: usize,
    pub is_captured: bool,
}

pub struct UserTypeInfo {
    pub id: TypeID,
}

pub struct RecordInfo {
    pub name: String,
    pub loc: Loc,
    pub scheme: Scheme,
    pub type_args: Option<Box<[RecordArgInfo]>>,
    pub fields: Box<[RecordFieldInfo]>,
}

pub struct RecordArgInfo {
    #[allow(dead_code)]
    pub name: Option<String>,
}

#[derive(Clone)]
pub struct RecordFieldInfo {
    pub name: String,
    pub ty: TypeID,
    pub loc: Loc,
}

pub struct UnionInfo {
    pub name: String,
    pub loc: Loc,
    pub scheme: Scheme,
    pub type_args: Option<Box<[UnionArgInfo]>>,
    pub variants: Box<[VariantInfo]>,
}

impl UnionInfo {
    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }
}

pub struct UnionArgInfo {
    #[allow(dead_code)]
    pub name: Option<String>,
}

pub struct VariantInfo {
    pub name: String,
    pub loc: Loc,
    pub expr: Expr,
    pub scheme: Scheme,
    pub type_args: Option<Box<[TypeID]>>,
}

impl VariantInfo {
    pub fn arity(&self) -> Option<usize> {
        self.type_args.as_ref().map(|args| args.len())
    }
}

pub struct ClassInfo {
    pub name: String,
    pub loc: Loc,
    pub items: Box<[ClassItemInfo]>,
    pub arity: (usize, usize),
}

pub struct ClassItemInfo {
    pub name: String,
    pub loc: Loc,
    pub scheme: Scheme,
}

#[derive(Clone)]
pub struct InstanceInfo {
    pub loc: Loc,
    pub scheme: InstanceScheme,
    pub original: InstanceID,
}

pub struct ImportInfo {
    pub name: String,
    pub loc: Loc,
    pub file: usize,
}

pub struct AliasInfo {
    pub name: String,
    pub path: PathQuery,
}

macro_rules! entity_impl {
    (
        $field:ident,
        ($create_name:ident, $next_id_name:ident, $get_name:ident $get_mut_name:ident),
        ($ID:ident => $Info:ty),
    ) => {
        pub fn $create_name(&mut self, info: $Info) -> $ID {
            let id = self.$next_id_name();
            self.$field.push(info);
            id
        }

        pub fn $next_id_name(&self) -> $ID {
            $ID(self.$field.len())
        }

        pub fn $get_name(&self, id: $ID) -> &$Info {
            &self.$field[id.0]
        }

        pub fn $get_mut_name(&mut self, id: $ID) -> &mut $Info {
            &mut self.$field[id.0]
        }
    };
}

macro_rules! define_id_type {
    ($ID:ident, $Variant:ident) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub struct $ID(pub usize);

        impl $ID {
            pub fn wrap(self) -> AnyID {
                AnyID::$Variant(self)
            }
        }
    };
}

use define_id_type;
use entity_impl;
