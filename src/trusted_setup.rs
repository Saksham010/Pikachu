use core::panic;
use std::{vec};
use ark_bn254::g1::Config;
use ark_ec::short_weierstrass::Projective;
use ark_ec::{AffineRepr, CurveGroup, Group};
use ark_ff::{Fp, MontBackend, PrimeField};
use std::fs::File;
use ark_bn254::{Fr as ScalarField, G1Projective as G};
use ark_std::{UniformRand};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial,Polynomial};
use ark_poly::{univariate::DenseOrSparsePolynomial};
use pikachu::{parse_circuit,compute_op_points,compute_op_polynomial,compute_vanishing_polynomial};
use ark_bn254::Fr;
use ark_serialize::CanonicalSerialize;
use std::io::prelude::*;
use std::io::Result;


fn save_key_to_file(key:Vec<Vec<Projective<Config>>>,file_name:&str) -> Result<()>{
    let mut file = File::create(file_name).unwrap();
    const DELIMITER:&[u8] = &[0];
    for vector in key {
        for element in vector {

            println!("Element: {:?}",element);
            println!("Element (X) : {:?}",element.x);
            println!("Element (Y) : {:?}",element.y);
            println!("Element (Z) : {:?}",element.z);


            let element_x = element.x;
            let element_y = element.y;
            let element_z = element.z;

            let mut serialized_data_x = Vec::new();
            let mut serialized_data_y = Vec::new();
            let mut serialized_data_z = Vec::new();

            element_x.serialize_uncompressed(&mut serialized_data_x).unwrap();
            element_y.serialize_uncompressed(&mut serialized_data_y).unwrap();
            element_z.serialize_uncompressed(&mut serialized_data_z).unwrap();

            let x_len: Vec<u8> = vec![serialized_data_x.len() as u8];
            let y_len: Vec<u8> = vec![serialized_data_y.len() as u8];
            let z_len: Vec<u8> = vec![serialized_data_z.len() as u8];

            println!("X_Data length: {:?}", &x_len);
            println!("Y_Data length: {:?}", &y_len);
            println!("Z_Data length: {:?}", &z_len);

            println!("Data to be stored (X): {:?}",&serialized_data_x);
            println!("Data to be stored (Y): {:?}",&serialized_data_y);
            println!("Data to be stored (Z): {:?}",&serialized_data_z);

            file.write_all(&x_len).unwrap();
            file.write_all(&mut serialized_data_x).unwrap();

            file.write_all(&y_len).unwrap();
            file.write_all(&mut serialized_data_y).unwrap();

            file.write_all(&z_len).unwrap();
            file.write_all(&mut serialized_data_z).unwrap();
        }
        file.write_all(DELIMITER)?; // Write delimiter after vector element
    }

    Ok(())
}

fn main() {
    let parsed_operations = parse_circuit("circuit.pika");
    println!("Operations: {:?}", parsed_operations);

    let left_op_points = compute_op_points(parsed_operations.clone(), 0);
    let right_op_points = compute_op_points(parsed_operations.clone(), 1);
    let ouput_op_points = compute_op_points(parsed_operations.clone(), 2);

    //Lagrange interpolation
    let (left_operand_polynomial_array,left_operand_polynomial) = compute_op_polynomial(left_op_points);
    let (right_operand_polynomial_array,right_operand_polynomial) = compute_op_polynomial(right_op_points);
    let (output_operand_polynomial_array,output_operand_polynomial) = compute_op_polynomial(ouput_op_points);

    let vanishing_p = compute_vanishing_polynomial(parsed_operations.len());

    //Sample random generator
    let mut rng = ark_std::test_rng();
    let g = G::generator(); //Generator on the curve

    let s:Fr = ScalarField::rand(&mut rng);
    
    let rohl:Fr = ScalarField::rand(&mut rng);
    let rohr:Fr = ScalarField::rand(&mut rng);
    let roho:Fr = rohl * rohr;
    let alphal:Fr = ScalarField::rand(&mut rng);
    let alphar:Fr = ScalarField::rand(&mut rng);
    let alphao:Fr = ScalarField::rand(&mut rng);
    let beta:Fr = ScalarField::rand(&mut rng);
    let gamma:Fr =  ScalarField::rand(&mut rng);
    let t_eval:Fr = vanishing_p.evaluate(&s);

    let gl = g*rohl;
    let gr = g*rohr;
    let go = g*roho;


    let mut gsk :Vec<Projective<Config>>= Vec::new(); //Proving key
    let mut gl_left_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key and Verification key
    let mut gr_right_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key and Verification key
    let mut go_output_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Porving key and Verification key

    let mut gl_alpha_left_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key
    let mut gr_alpha_right_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key
    let mut go_alpha_output_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Porving key

    let mut gl_beta_left_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key
    let mut gr_beta_right_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key
    let mut go_beta_output_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Proving key

    let gl_t_eval = gl * t_eval; //Proving key
    let gr_t_eval = gr * t_eval; //Proving key
    let go_t_eval = go * t_eval; //Proving key and Verification key

    let gl_alphal_t_eval = gl * (alphal*t_eval); //Proving key
    let gr_alphar_t_eval = gr * (alphar*t_eval); //Proving key
    let go_alphao_t_eval = go * (alphao*t_eval); //Proving key

    let gl_beta_t_eval = gl * (beta*t_eval); //Proving key
    let gr_beta_t_eval = gr * (beta*t_eval); //Proving key
    let go_beta_t_eval = go * (beta*t_eval); //Proving key

    let g_alphal = g * alphal; //Verification key
    let g_alphar = g * alphar; //Verification key
    let g_alphao = g * alphao; //Verification key
    let g_gamma = g * gamma; //Verification key
    let g_beta_gamma = g * (beta*gamma); //Verification key


    //Compute g^s^k for 0<= k <= no of operations
    for (i,_) in (0..parsed_operations.len()).into_iter().enumerate(){
        let gi = (g*s)*ScalarField::from((i+1) as u64);
        gsk.push(gi);
    }

    //Compute evaluations : gl^li(s) , gl^alphal*li(s) , gl^beta*li(s) 
    for poly in left_operand_polynomial_array {
        let eval:Fr = poly.evaluate(&s);
    
        let gl_alphal_li = gl * (alphal*eval);
        let gl_beta_li = gl * (beta*eval);
        let gl_li = gl*eval;

        gl_alpha_left_operand_poly_eval.push(gl_alphal_li);
        gl_beta_left_operand_poly_eval.push(gl_beta_li);
        gl_left_operand_poly_eval.push(gl_li);
    }

    //Compute evaluations : gr^ri(s) , gr^alphar*ri(s) ,  gr^beta*ri(s) 
    for poly in right_operand_polynomial_array {
        let eval:Fr = poly.evaluate(&s);

        let gr_alphar_ri = gr * (alphar*eval);
        let gr_beta_ri = gr * (beta*eval);
        let gr_ri = gr*eval;

        gr_alpha_right_operand_poly_eval.push(gr_alphar_ri);
        gr_beta_right_operand_poly_eval.push(gr_beta_ri);
        gr_right_operand_poly_eval.push(gr_ri);

    }

    //Compute evaluations : go^oi(s) , go^alphao*oi(s) ,  go^beta*oi(s) 
    for poly in output_operand_polynomial_array {
        let eval:Fr = poly.evaluate(&s);

        let go_alphar_oi = go * (alphao*eval);
        let go_beta_oi = go * (beta*eval);
        let go_oi = go*eval;

        go_alpha_output_operand_poly_eval.push(go_alphar_oi);
        go_beta_output_operand_poly_eval.push(go_beta_oi);
        go_output_operand_poly_eval.push(go_oi);

    }

    // println!("Generator: {:?}",g);
    // println!("Secret s: {:?}",s);
    // println!("Rohl: {:?}",rohl);
    // println!("Rohr: {:?}",rohr);
    // println!("Roho: {:?}",roho);
    // println!("alphal: {:?}",alphal);
    // println!("alphar: {:?}",alphar);
    // println!("alphao: {:?}",alphao);
    // println!("beta: {:?}",beta);
    // println!("gamma: {:?}",gamma);
    // println!("gl: {:?}",gl);
    // println!("gr: {:?}",gr);
    // println!("go: {:?}",go);


    // Serialize proving and verification key to bytes and save them in a file
    
    // Provking key part 2
    let pk_2: Vec<Projective<Config>> = vec![
        gl_t_eval,
        gr_t_eval,
        go_t_eval,
        gl_alphal_t_eval,
        gr_alphar_t_eval,
        go_alphao_t_eval,
        gl_beta_t_eval,
        gr_beta_t_eval,
        go_beta_t_eval,
        g_alphal,
        g_alphar,
        g_alphao,
        g_gamma,
        g_beta_gamma
    ];

    //Final proving key
    let proving_key:Vec<Vec<Projective<Config>>> = vec![
        gsk.clone(),
        gl_left_operand_poly_eval.clone(),
        gr_right_operand_poly_eval.clone(),
        go_output_operand_poly_eval.clone(),
        gl_alpha_left_operand_poly_eval,
        gr_alpha_right_operand_poly_eval,
        go_alpha_output_operand_poly_eval,
        gl_beta_left_operand_poly_eval,
        gr_beta_right_operand_poly_eval,
        go_beta_output_operand_poly_eval,
        pk_2
    ];


    // Verification key part 2
    let vk_2:Vec<Projective<Config>> = vec![
        go_t_eval,
        g_alphal,
        g_alphar,
        g_alphao,
        g_gamma,
        g_beta_gamma
    ];

    //Final verification key
    let verification_key:Vec<Vec<Projective<Config>>> = vec![
        gl_left_operand_poly_eval,
        gr_right_operand_poly_eval,
        go_output_operand_poly_eval,
        vk_2        
    ];


    println!("Proving key: {:?}",proving_key);
    println!("Verification key: {:?}",verification_key);

    // Serialize and save
    let mut res = save_key_to_file(proving_key, "proving_key.bin"); // Save proving key

    match &res {
        Ok(()) => {
            println!("Proving key generated !!");
        }
        Err(msg)=>{
            println!("Failed to generate proving key: {:?}",msg);
        }
    }

    res = save_key_to_file(verification_key, "verification_key.bin"); // Save verification key

    match &res {
        Ok(()) => {
            println!("Verification key generated !!");
        }
        Err(msg)=>{
            println!("Failed to generate verification key: {:?}",msg);
        }
    }

}
