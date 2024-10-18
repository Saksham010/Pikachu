use ark_ff::{PrimeField,Zero};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial};
use std::{ops::Mul, vec};
use std::path::Path;
use std::fs::File;
use ark_bn254::Fr;
use std::io::Read;


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

pub fn shield_brack_parser(op: &str) -> (String, String) {
    let op_arr: Vec<char> = op.chars().collect();
    let digits: String = op.chars().filter(|c: &char| c.is_numeric()).collect();
    let operand: String = op.chars().filter(|c| c.is_alphabetic()).collect();

    if op_arr[0] == '[' {
        let sub = '-';
        let ref_digits = &digits;
        let coeff = sub.to_string() + ref_digits;
        return (coeff, operand);
    } else {
        return (digits, operand);
    }
}

pub fn parse_circuit(file_name:&str) -> Vec<[String; 5]> {
    let path = Path::new(file_name);
    let file = File::open(path);
    let mut contents = String::new();
    let res = file.expect("Unable to open").read_to_string(&mut contents);
    let mut shield_brack: bool = false;
    let mut parsed_operations: Vec<[String; 5]> = Vec::new();

    //Handle error
    match res {
        Ok(_) => {
            println!("Analyzing circuit");
        }
        Err(_) => {
            panic!("Circuit analysis failed");
        }
    }

    //Remove whitespace
    let operations: Vec<String> = contents
        .split("\r\n")
        .filter(|o| *o != "")
        .map(|op: &str| op.chars().filter(|c: &char| !c.is_whitespace()).collect())
        .collect();

    let _supported_operation = ['*', '+', '-', '/'];
    let _supported_operation_string = "*+-/";

    operations.iter().for_each(|op| {
        let mut sp_idx = None;
        let idx: usize;

        let op_count = op
            .chars()
            .into_iter()
            .filter(|c| {
                //Don't count operator inside sqbracket
                if *c == '[' {
                    shield_brack = true;
                } else if *c == ']' {
                    shield_brack = false;
                }

                if !shield_brack {
                    _supported_operation_string.contains(*c)
                } else {
                    return false;
                }
            })
            .count();

        // println!("Total operator count in the current operation: {}",op_count);

        match op_count {
            0 => {
                panic!("ERROR: Missing operand in circuit");
            }
            1 => {
                //Find operator index
                for sp in _supported_operation {
                    sp_idx = op.find(sp);
                    match sp_idx {
                        Some(_) => break,
                        None => {}
                    }
                }

                match sp_idx {
                    Some(_idx) => idx = _idx,
                    None => {
                        panic!("ERROR: Unsupported operation")
                    }
                }

                //Use operator index to split to 3 parts ie left operand,right operand and result
                let (leftoperand, r) = op.split_at(idx);
                let right = &r[1..];
                let parts: Vec<&str> = right.split("==").collect();
                let rightoperand = parts[0];
                let output = parts[parts.len() - 1];

                let (lcoeff, lop) = shield_brack_parser(leftoperand);
                let (rcoeff, rop) = shield_brack_parser(rightoperand);

                //Check if any operand is empty
                if lop.is_empty() || rop.is_empty() || output.is_empty() {
                    panic!("ERROR: Invalid operands or output in circuit")
                }

                let val_vec: [String; 5] = [lcoeff, lop, rcoeff, rop, String::from(output)];
                // println!("Val Vec: {:?}", val_vec);

                //Borrow value
                parsed_operations.push(val_vec);
            }
            _ => {
                panic!("ERROR: More than one operand in circuit");
            }
        }
    });

    println!("Circuit analyzed");
    return parsed_operations;
}

pub fn compute_op_points(parsed_operations: Vec<[String; 5]>, op_type: i32) -> (Vec<Vec<[i32; 2]>>,Vec<String>) {
    let mut op_points_list: Vec<Vec<[i32; 2]>> = Vec::new();

    let offset = if op_type == 0 {
        0
    } else if op_type == 1 {
        2
    } else {
        4
    };

    // Variable ocurrance map
    let mut occurance_list:Vec<String> = Vec::new();
    for array in &parsed_operations{
        let mut exist = false;
        // println!("Occurance list currently: {:?}",&occurance_list);
        for op_var in &occurance_list{
            
            if op_type == 2{

                 if op_var == &array[offset] {
                     exist = true;
                 }
            }else {
                if op_var == &array[offset+1]{
                    exist = true;
                }
            }
        }
        if !exist {
            if op_type == 2{
                occurance_list.push(array[offset].clone());
            }else{
                occurance_list.push(array[offset+1].clone());
            }
        }
    }

    // println!("Occurance list: {:?}",occurance_list);

    let mut x_index:i32 = 1;

    for oc_var in &occurance_list{
        let mut inner_vec: Vec<[i32; 2]> = Vec::new();

        for array in &parsed_operations{

            let coeff: &str = if &array[0 + offset] == "" || op_type == 2 {
                "1"
            } else {
                &array[0 + offset]
            }; 

    
            let op_var:&String = if op_type == 2 {
                &array[offset]
            } else {
                &array[offset+1]
            };
    
            let x: i32 = x_index;
            let y:i32 = coeff.parse().expect("Not a valid number");

            // println!("[X,Y]: [{:?},{:?}]",x,y);
            if oc_var == op_var{
                inner_vec.push([x,y]);
                
            } else {
                inner_vec.push([x,0]);
            }
            x_index = x_index + 1;
        }

        //Add inner vec to op list
        op_points_list.push(inner_vec.clone());
        inner_vec.clear();

        //Reset x for another occurance
        x_index = 1;
    }
    // println!("Operand points: {:?}", op_points_list);
    return (op_points_list,occurance_list);
}

pub fn compute_op_polynomial(op_points: Vec<Vec<[i32; 2]>>) ->(Vec<DensePolynomial<Fr>>,DensePolynomial<Fr>) {

    let mut polynomial_array:Vec<DensePolynomial<Fr>> = Vec::new();
    let mut final_polynomial:DensePolynomial<Fr> = DensePolynomial::from_coefficients_vec(vec![Fr::zero()]);

    for points in &op_points{
        //First variable polynomial
        let mut x_point_list:Vec<i32> = Vec::new();
        let mut y_point_list:Vec<i32> = Vec::new();
    
        //Seperate x and y point list
        for point in points{
            
            let x = point[0];
            let y = point[1];
                    
            x_point_list.push(x);
            y_point_list.push(y);
            
        }

        let x_point_list_u64:Vec<u64> = x_point_list.iter().map(|&p| p as u64).collect();
        let y_point_list_u64:Vec<u64>= y_point_list.iter().map(|&p| p as u64).collect();

        let pair_point_list: Vec<(Fr, Fr)> = x_point_list_u64.iter()
        .zip(y_point_list_u64.iter())
        .map(|(&x, &y)| (Fr::from(x), Fr::from(y)))
        .collect();

        //Interpolate polynomial from those points
        let c0_polynomial = lagrange_interpolation_polynomial(&pair_point_list);
        polynomial_array.push(c0_polynomial);
    }

    //Compute final polynomial
    for poly in &polynomial_array{
        final_polynomial = &final_polynomial + poly;
    }
    return (polynomial_array,final_polynomial);


}

pub fn compute_vanishing_polynomial(length:usize) -> DensePolynomial<Fr> {
    
    let mut vanishing_index:u64 = 0;    
    let mut vanishing_list:Vec<[Fr;2]> = Vec::new();
    let mut count = 0;
    loop{
        if count == length {
            break;
        }
        vanishing_index = vanishing_index + 1;

        //Subtract 0 with vanishing index field we get its negative under the prime field
        let neg_vanishing_index_in_prime_field = Fr::zero() - Fr::from(vanishing_index);

        vanishing_list.push([neg_vanishing_index_in_prime_field,Fr::from(1u64)]);
        count += 1;
    }


    let mut vanishing_p:DensePolynomial<Fr> = DensePolynomial::from_coefficients_vec(vec![Fr::from(1u64)]); 

    for l in vanishing_list{
        vanishing_p = vanishing_p.mul(&DensePolynomial::from_coefficients_vec(l.to_vec()));
    }

    vanishing_p
    
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
