use core::panic;
use std::{vec};
use ark_bn254::g1::Config;
use ark_ec::short_weierstrass::Projective;
use std::fs::File;
use std::io::{Cursor, Read};
use ark_bn254::{Fq, FqConfig, Fr as ScalarField, G1Projective as G};
use ark_std::{Zero, UniformRand, ops::Mul,ops::Sub};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial,Polynomial};
use ark_poly::{univariate::DenseOrSparsePolynomial};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use std::io::prelude::*;
use std::io::Result;
use ark_ff::{Fp, MontBackend};
use pikachu::{parse_circuit,compute_op_points,compute_op_polynomial,compute_vanishing_polynomial};


const DELIMITER: &[u8] = &[0]; // Inner delimiter for separating vec of elements

fn load_key_from_file(file_name:&str) -> Result<Vec<Vec<Projective<Config>>>>{
    let mut file = File::open(file_name).unwrap();

    let mut final_key:Vec<Vec<Projective<Config>>> = Vec::new();
    let mut key_vec_inner:Vec<Projective<Config>> = Vec::new();

    //Buffer to load the file
    let mut buffer:Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    // let mut cursor = &buffer[..];
    let mut cursor = Cursor::new(&buffer[..]);

    // Deserialize each values
    while (cursor.position() as usize) < cursor.get_ref().len(){ //Ignoring the last 0 delimiter
        //Read the length
        let mut element_len =[0u8];
        cursor.read_exact(&mut element_len).unwrap(); // Read x length

        //If delimeter found
        if element_len == DELIMITER{
            //Push and clear the inner vec
            final_key.push(key_vec_inner.clone());
            key_vec_inner.clear();

            //If last index continue/break
            if cursor.position() as usize == cursor.get_ref().len() {
                continue;
            }            
            //Else next element length
            cursor.read_exact(&mut element_len).unwrap(); // Read again length
        }

        let mut x_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut x_element).unwrap(); //Read x 

        cursor.read_exact(&mut element_len).unwrap(); // Read y length
        let mut y_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut y_element).unwrap(); //Read y 

        cursor.read_exact(&mut element_len).unwrap(); // Read z length
        let mut z_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut z_element).unwrap(); //Read z

        //Deseralize
        let mut cursorx = Cursor::new(x_element);
        let mut cursory = Cursor::new(y_element);
        let mut cursorz = Cursor::new(z_element);

        let deserialized_x:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorx).unwrap();
        let deserialized_y:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursory).unwrap();
        let deserialized_z:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorz).unwrap();

        let element:G = G::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check

        key_vec_inner.push(element); //Push the element
    }

    println!("{:?}: {:?}",file_name,final_key);
    Ok(final_key)
    
}

fn main(){

    let proving_key = load_key_from_file("proving_key.bin").unwrap();
    let verification_key = load_key_from_file("verification_key.bin").unwrap();

    let parsed_operations = parse_circuit("./src/prover/prover_polynomial.pika");
    println!("Operations: {:?}", parsed_operations);

    let left_op_points = compute_op_points(parsed_operations.clone(), 0);
    let right_op_points = compute_op_points(parsed_operations.clone(), 1);
    let ouput_op_points = compute_op_points(parsed_operations.clone(), 2);

    //Lagrange interpolation
    let (left_operand_polynomial_array,left_operand_polynomial) = compute_op_polynomial(left_op_points);
    let (right_operand_polynomial_array,right_operand_polynomial) = compute_op_polynomial(right_op_points);
    let (output_operand_polynomial_array,output_operand_polynomial) = compute_op_polynomial(ouput_op_points);

    let vanishing_p = compute_vanishing_polynomial(parsed_operations.len());


    // --- Test --- Start
    // let a = Fr::from(1u64);
    // let b = Fr::from(2u64);
    // let c = Fr::from(1u64);
    // let r1 = Fr::from(12u64);
    // let r2 = Fr::from(1u64);

    // println!("LEFTOPARRAY: {:?}",left_operand_polynomial_array);
    // println!("LEFTOP: {:?}",left_operand_polynomial);

    // let final_left_polynomial = left_operand_polynomial.mul(&DensePolynomial::from_coefficients_vec(vec![a,Fr::zero()]));
    // let mut final_right_polynomial = DensePolynomial::from_coefficients_vec(vec![Fr::zero()]);
    // let mut final_out_polynomial = DensePolynomial::from_coefficients_vec(vec![Fr::zero()]);


    // for (i,poly) in right_operand_polynomial_array.iter().enumerate() {
    //     if i == 0 {
    //         final_right_polynomial =  final_right_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![b,Fr::zero()]));
    //     }else if i == 1{
    //         final_right_polynomial =  final_right_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![c,Fr::zero()]));
    //     }
    // }

    // for (i,poly) in output_operand_polynomial_array.iter().enumerate() {
    //     if i == 0 {
    //         final_out_polynomial =  final_out_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![r1,Fr::zero()]));
    //     }else if i == 1{
    //         final_out_polynomial =  final_out_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![r2,Fr::zero()]));
    //     }
    // }

    
    // let polynomial_p = &final_left_polynomial.mul(&final_right_polynomial) - &final_out_polynomial;
    // let vanishing_p = compute_vanishing_polynomial(parsed_operations.len());
    // let (qoutient,remainder)= DenseOrSparsePolynomial::from(polynomial_p.clone()).divide_with_q_and_r(&DenseOrSparsePolynomial::from(vanishing_p.clone())).unwrap();

    // println!("Quotient: {:?}",qoutient);
    // println!("Remainder: {:?}",remainder);

    // TEST --- Complete


}