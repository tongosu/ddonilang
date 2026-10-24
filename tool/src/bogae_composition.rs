//! RD-130/132/133/520의 공통 보개짜임 구조 검사.
//!
//! Identity, 보기값, 공개 대상, 봉인 보기법은 해소기가 공급하는 typed 값이다.
//! 이 모듈은 그 값의 source spelling, 정본 bytes 또는 입력 파일 형식을 만들지 않는다.
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationKind<I, V, T, R> {
    Primary { view: V },
    Overlay { view: V, basis: I, order: u64 },
    Side { view: V, placement: SidePlacement },
    Magnifier { target: T, recipe: R },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidePlacement {
    // RD-134: authored placement hint, not a renderer-selected pixel position.
    Auto,
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredRelation<I, V, T, R> {
    pub identity: I,
    pub kind: RelationKind<I, V, T, R>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositionError {
    PrimaryCount,
    RelationDuplicate,
    OverlayOrderDuplicate,
    OverlayBasisInvalid,
}

impl CompositionError {
    pub fn code(self) -> &'static str {
        match self {
            Self::PrimaryCount => "E_BOGAE_PRIMARY_COUNT",
            Self::RelationDuplicate => "E_BOGAE_RELATION_DUPLICATE",
            Self::OverlayOrderDuplicate => "E_BOGAE_OVERLAY_ORDER_DUPLICATE",
            Self::OverlayBasisInvalid => "E_BOGAE_OVERLAY_BASIS_INVALID",
        }
    }
}

/// 검사된 authored list만 보존한다. 보기 hash로 관계를 합치거나 순서를 바꾸지 않는다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition<I, V, T, R> {
    relations: Vec<AuthoredRelation<I, V, T, R>>,
    primary_index: usize,
}

impl<I: Ord, V, T, R> Composition<I, V, T, R> {
    pub fn resolve(relations: Vec<AuthoredRelation<I, V, T, R>>) -> Result<Self, CompositionError> {
        let mut identities = BTreeSet::new();
        let mut primary_index = None;
        for (index, relation) in relations.iter().enumerate() {
            if !identities.insert(&relation.identity) {
                return Err(CompositionError::RelationDuplicate);
            }
            if matches!(relation.kind, RelationKind::Primary { .. }) {
                if primary_index.replace(index).is_some() {
                    return Err(CompositionError::PrimaryCount);
                }
            }
        }
        let primary_index = primary_index.ok_or(CompositionError::PrimaryCount)?;
        let primary_identity = &relations[primary_index].identity;
        let mut overlay_orders = BTreeSet::new();
        for relation in &relations {
            if let RelationKind::Overlay { basis, order, .. } = &relation.kind {
                if basis != primary_identity {
                    return Err(CompositionError::OverlayBasisInvalid);
                }
                if !overlay_orders.insert(*order) {
                    return Err(CompositionError::OverlayOrderDuplicate);
                }
            }
        }
        Ok(Self {
            relations,
            primary_index,
        })
    }

    pub fn relations(&self) -> &[AuthoredRelation<I, V, T, R>] {
        &self.relations
    }

    pub fn primary(&self) -> &AuthoredRelation<I, V, T, R> {
        &self.relations[self.primary_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type Relation = AuthoredRelation<u32, u32, (), ()>;
    fn primary(identity: u32) -> Relation {
        Relation {
            identity,
            kind: RelationKind::Primary { view: 9 },
        }
    }
    fn overlay(identity: u32, basis: u32, order: u64) -> Relation {
        Relation {
            identity,
            kind: RelationKind::Overlay {
                view: 9,
                basis,
                order,
            },
        }
    }

    #[test]
    fn requires_exactly_one_primary() {
        assert_eq!(
            Composition::<u32, u32, (), ()>::resolve(vec![]),
            Err(CompositionError::PrimaryCount)
        );
        assert_eq!(
            Composition::resolve(vec![overlay(2, 1, 0)]),
            Err(CompositionError::PrimaryCount)
        );
        assert_eq!(
            Composition::resolve(vec![primary(1), primary(2)]),
            Err(CompositionError::PrimaryCount)
        );
    }

    #[test]
    fn identical_views_keep_distinct_relations_and_authored_order() {
        let input = vec![overlay(2, 1, 7), primary(1), overlay(3, 1, 2)];
        let composition = Composition::resolve(input.clone()).unwrap();
        assert_eq!(composition.relations(), input);
        assert_eq!(composition.primary().identity, 1);
    }

    #[test]
    fn rejects_duplicate_identity_even_with_different_roles() {
        assert_eq!(
            Composition::resolve(vec![primary(1), overlay(1, 1, 0)]),
            Err(CompositionError::RelationDuplicate)
        );
    }

    #[test]
    fn rejects_foreign_overlay_basis_and_duplicate_order() {
        assert_eq!(
            Composition::resolve(vec![primary(1), overlay(2, 99, 0)]),
            Err(CompositionError::OverlayBasisInvalid)
        );
        assert_eq!(
            Composition::resolve(vec![primary(1), overlay(2, 2, 0)]),
            Err(CompositionError::OverlayBasisInvalid)
        );
        assert_eq!(
            Composition::resolve(vec![primary(1), overlay(2, 1, 4), overlay(3, 1, 4)]),
            Err(CompositionError::OverlayOrderDuplicate)
        );
    }

    #[test]
    fn preserves_all_authored_side_placement_hints() {
        for placement in [SidePlacement::Auto, SidePlacement::Left,
            SidePlacement::Right, SidePlacement::Top, SidePlacement::Bottom] {
            let side = Relation {
                identity: 2,
                kind: RelationKind::Side { view: 9, placement },
            };
            let composition = Composition::resolve(vec![primary(1), side.clone()]).unwrap();
            assert_eq!(composition.relations()[1], side);
        }
    }
}
