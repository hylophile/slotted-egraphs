use crate::*;

use std::hash::*;

// Should ProvenNode also contain the src-id?
#[derive(Clone)]
pub struct ProvenNode<L> {
    pub elem: L,

    #[cfg(feature = "explanations_tmp")]
    pub proofs: Vec<ProvenEq>,
}

impl<L: Language> PartialEq for ProvenNode<L> {
    fn eq(&self, other: &Self) -> bool { self.elem == other.elem }
}

impl<L: Language> Eq for ProvenNode<L> { }

impl<L: Language> Hash for ProvenNode<L> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.elem.hash(hasher);
    }
}

impl<L: Language> ProvenNode<L> {
    // checks that `proofs` brings us from `base` to `elem`.
    #[cfg(feature = "explanations_tmp")]
    pub fn check_base(&self, base: &L) {
        let l = base.applied_id_occurences();
        let r = self.elem.applied_id_occurences();
        let n = self.proofs.len();
        assert_eq!(n, l.len());
        assert_eq!(n, r.len());
        for i in 0..n {
            let l = l[i].clone();
            let r = r[i].clone();
            let eq = Equation { l, r };
            assert_proves_equation(&self.proofs[i], &eq);
        }
    }

    pub fn weak_shape(&self) -> (Self, Bijection) {
        let (sh, bij) = self.elem.weak_shape();
        let pn = ProvenNode {
            elem: sh,

            #[cfg(feature = "explanations_tmp")]
            proofs: self.proofs.clone(),
        };
        (pn, bij)
    }
}

impl<L: Language> EGraph<L> {
    pub fn refl_pn(&self, start: &L) -> ProvenNode<L> {
        #[cfg(feature = "explanations_tmp")]
        let rfl = start.applied_id_occurences()
                       .into_iter()
                       .map(|x| self.refl_proof(x.id))
                       .collect();
        ProvenNode {
            elem: start.clone(),
            #[cfg(feature = "explanations_tmp")]
            proofs: rfl,
        }
    }

    #[cfg(feature = "explanations_tmp")]
    fn refl_proof(&self, i: Id) -> ProvenEq {
        let syn_slots = self.syn_slots(i);
        let identity = SlotMap::identity(&syn_slots);
        let app_id = AppliedId::new(i, identity);
        self.prove_reflexivity(&app_id)
    }

    pub fn chain_pn_map(&self, start: &ProvenNode<L>, f: impl Fn(usize, ProvenAppliedId) -> ProvenAppliedId) -> ProvenNode<L> {
        let mut pn = start.clone();
        let n = pn.elem.applied_id_occurences().len();

        let mut app_ids_mut: Vec<&mut AppliedId> = pn.elem.applied_id_occurences_mut();

        #[cfg(feature = "explanations_tmp")]
        let mut proofs_mut: &mut [ProvenEq] = &mut pn.proofs;

        for i in 0..n {
            let old_app_id: &mut AppliedId = app_ids_mut[i];
            #[cfg(feature = "explanations_tmp")]
            let old_proof: &mut ProvenEq = &mut proofs_mut[i];

            let tmp_pai = ProvenAppliedId {
                elem: old_app_id.clone(),
                #[cfg(feature = "explanations_tmp")]
                proof: old_proof.clone(),
            };
            let pai = f(i, tmp_pai);

            *old_app_id = pai.elem;

            #[cfg(feature = "explanations_tmp")]
            { *old_proof = pai.proof; }
        }
        pn
    }
}
