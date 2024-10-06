use ark_ff::{PrimeField};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use std::{ops::{Mul}, vec};


pub fn lagrange_interpolation_polynomial<F: PrimeField>(points: &[(F, F)]) -> DensePolynomial<F> {
    let zero = DensePolynomial::from_coefficients_vec(vec![F::zero()]);
    let one = DensePolynomial::from_coefficients_vec(vec![F::one()]);

    points.iter().enumerate().fold(zero, |acc, (i, &(xi, yi))| {
        let mut term = one.clone();
        let mut denominator: F = F::one();

        for (j, &(xj, _)) in points.iter().enumerate() {
            if i != j {
                term = term.mul(&DensePolynomial::from_coefficients_vec(vec![-xj, F::one()]));
                denominator *= xi - xj;
            }
        }

        
        let scalar: F = yi * denominator.inverse().unwrap();
        acc + term.mul(&DensePolynomial::from_coefficients_vec(vec![scalar,F::zero()]))
    })

}


#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_poly::Polynomial;

    #[test]
    fn lagrange_interpolation_test() {
        //Dummy data    
        let x0 = Fr::from(1u64);
        let y0 = Fr::from(2u64);

        let x1 = Fr::from(2u64);
        let y1 = Fr::from(3u64);

        let x2 = Fr::from(3u64);
        let y2 = Fr::from(5u64);


        let points:Vec<(Fr,Fr)> = vec![(x0,y0),(x1,y1),(x2,y2)];

        println!("Points in BN254 scalar field:");
        for (_, (x, y)) in points.iter().enumerate() {
            println!("({}, {})", x, y);
        }

        let polynomial = lagrange_interpolation_polynomial(&points);


        println!("\nInterpolation polynomial coefficients:");
        for (i, coeff) in polynomial.coeffs.iter().enumerate() {
            println!("Coefficient of x^{}: {}", i, coeff);
        }

        println!("Polynomial: {:?}",polynomial);



        //Testing data:
        println!("\nVerification:");
        let tx1 = Fr::from(4u64);
        let ty1= Fr::from(8u64);

        let tx2 = Fr::from(5u64);
        let ty2= Fr::from(12u64);

        let tx3 = Fr::from(6u64);
        let ty3= Fr::from(17u64);


        let testing_data = vec![(x0,y0),(x1,y1),(x2,y2),(tx1,ty1),(tx2,ty2),(tx3,ty3)];
        for (x,y) in &testing_data {
            let y_interp = polynomial.evaluate(x);
            println!("P({}) = {} (original y: {})", x, y_interp, y);
            assert_eq!(*y, y_interp, "Interpolation failed for x = {}", x);

        }
        println!("All points verified successfully!");
    }
}
