use super::circuit::{Any, Column};
use crate::{
    arithmetic::CurveAffine,
    helpers::CurveRead,
    poly::{Coeff, ExtendedLagrangeCoeff, LagrangeCoeff, Polynomial},
};

pub(crate) mod keygen;
pub(crate) mod prover;
pub(crate) mod verifier;

use std::io;

/// A permutation argument.
#[derive(Debug, Clone)]
pub struct Argument {
    /// A sequence of columns involved in the argument.
    pub columns: Vec<Column<Any>>,
}

impl Argument {
    pub(crate) fn new() -> Self {
        Argument { columns: vec![] }
    }

    /// Returns the minimum circuit degree required by the permutation argument.
    /// The argument may use larger degree gates depending on the actual
    /// circuit's degree and how many columns are involved in the permutation.
    pub(crate) fn required_degree(&self) -> usize {
        // degree 2:
        // l_0(X) * (1 - z(X)) = 0
        //
        // We will fit as many polynomials p_i(X) as possible
        // into the required degree of the circuit, so the
        // following will not affect the required degree of
        // this middleware.
        //
        // (1 - (l_last(X) + l_blind(X))) * (
        //   z(\omega X) \prod (p(X) + \beta s_i(X) + \gamma)
        // - z(X) \prod (p(X) + \delta^i \beta X + \gamma)
        // )
        //
        // On the first sets of columns, except the first
        // set, we will do
        //
        // l_0(X) * (z(X) - z'(\omega^(last) X)) = 0
        //
        // where z'(X) is the permutation for the previous set
        // of columns.
        //
        // On the final set of columns, we will do
        //
        // degree 3:
        // l_last(X) * (z'(X)^2 - z'(X)) = 0
        //
        // which will allow the last value to be zero to
        // ensure the argument is perfectly complete.

        // There are constraints of degree 3 regardless of the
        // number of columns involved.
        3
    }

    pub(crate) fn add_column(&mut self, column: Column<Any>) {
        if !self.columns.contains(&column) {
            self.columns.push(column);
        }
    }

    pub(crate) fn get_columns(&self) -> Vec<Column<Any>> {
        self.columns.clone()
    }
}

/// The verifying key for a single permutation argument.
#[derive(Debug, Clone)]
pub struct VerifyingKey<C: CurveAffine> {
    pub commitments: Vec<C>,
}

impl<C: CurveAffine> VerifyingKey<C> {
    pub fn write<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        for commitment in &self.commitments {
            writer.write_all(commitment.to_bytes().as_ref())?;
        }

        Ok(())
    }

    pub fn read<R: io::Read>(reader: &mut R, argument: &Argument) -> io::Result<Self> {
        let commitments = (0..argument.columns.len())
            .map(|_| C::read(reader))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(VerifyingKey { commitments })
    }
}

/// The proving key for a single permutation argument.
#[derive(Debug)]
pub(crate) struct ProvingKey<C: CurveAffine> {
    permutations: Vec<Polynomial<C::Scalar, LagrangeCoeff>>,
    pub(super) polys: Vec<Polynomial<C::Scalar, Coeff>>,

    #[cfg(not(feature = "cuda"))]
    pub(super) cosets: Vec<Polynomial<C::Scalar, ExtendedLagrangeCoeff>>,
}
