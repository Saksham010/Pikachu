use core::panic;
use std::{vec};
use ark_bn254::g1::Config;
use ark_bn254::g2::Config as Config2;
use ark_ec::bn::G1Projective;
use ark_ec::{pairing, CurveGroup, Group}; // For the `.mul()` method
use ark_ec::short_weierstrass::Projective;
use ark_poly::polynomial;
use std::fs::File;
use std::io::{Cursor, Read,BufReader};
use serde_json::{Value};
use std::collections::HashMap;
use ark_bn254::{Bn254, FqConfig,Fq2Config, Fr as ScalarField, FrConfig, G1Projective as G, G2Projective as G2};
use ark_std::{Zero, UniformRand, ops::Mul,ops::Sub};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial,Polynomial};
use ark_poly::{univariate::DenseOrSparsePolynomial};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use std::io::prelude::*;
use std::io::Result;
use ark_ff::{Fp,Fp2,Fp2ConfigWrapper,QuadExtField, MontBackend};
use pikachu::{parse_circuit,compute_op_points,compute_op_polynomial,compute_vanishing_polynomial};
use ark_ec::pairing::{Pairing,PairingOutput};

//For G1Projective and G2 projective coordinates
#[derive(Debug)]
#[derive(Clone)]
enum ProjectiveCoordinateType{
    C1(Fp<MontBackend<FqConfig, 4>, 4>),
    C2(QuadExtField<Fp2ConfigWrapper<Fq2Config>>)
}


//For G1Projective and G2 projective elements
#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
enum ProjectiveConfigType {
    GOne(Projective<Config>),
    GTwo(Projective<Config2>)
}

const DELIMITER: &[u8] = &[0]; // Inner delimiter for separating vec of elements

fn load_key_from_file(file_name:&str,g1g2_seperator_index:u8) -> Result<Vec<Vec<ProjectiveConfigType>>>{
    let mut file = File::open(file_name).unwrap();

    let mut final_key:Vec<Vec<ProjectiveConfigType>> = Vec::new();
    let mut key_vec_inner:Vec<ProjectiveConfigType> = Vec::new();

    //Buffer to load the file
    let mut buffer:Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    // let mut cursor = &buffer[..];
    let mut cursor = Cursor::new(&buffer[..]);

    let mut iteration_no = 0;

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

        iteration_no = iteration_no+1;

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


        if iteration_no >= g1g2_seperator_index {
            //For G2 elements in verification key
            let deserialized_x:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursorx).unwrap();
            let deserialized_y:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursory).unwrap();
            let deserialized_z:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursorz).unwrap();
    
            let element:Projective<Config2> = G2::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check
           
            key_vec_inner.push(ProjectiveConfigType::GTwo(element)); //Push the element
    
            println!("Iteration no: {:?}",iteration_no);
            println!("Element: {:?}",element);
        }else{
    
            let deserialized_x:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorx).unwrap();
            let deserialized_y:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursory).unwrap();
            let deserialized_z:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorz).unwrap();
    
            let element:Projective<Config> = G::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check
    
            key_vec_inner.push(ProjectiveConfigType::GOne(element)); //Push the element
    
            println!("Iteration no: {:?}",iteration_no);
            println!("Element: {:?}",element);

        }

    }

    println!("{:?}: {:?}",file_name,final_key);
    Ok(final_key)
    
}


fn load_witness_values() -> Result<HashMap<String,Value>>{
    let file = File::open("./src/prover/witness.json").unwrap();
    let reader = BufReader::new(file);
    let witness_values:HashMap<String,Value> = serde_json::from_reader(reader).unwrap();
    Ok(witness_values)
}

fn compute_final_polynomial(witness_values:HashMap<String,Value>,polynomial_array:Vec<DensePolynomial<Fp<MontBackend<FrConfig,4>,4>>>,occurance_list:Vec<String>) -> DensePolynomial<Fp<MontBackend<FrConfig,4>,4>>{
    
    //Compute operand polynomial
    let mut final_polynomial:DensePolynomial<Fp<MontBackend<FrConfig,4>,4>> = DensePolynomial::from_coefficients_vec(vec![ScalarField::zero()]);

    polynomial_array.clone().iter()
    .zip(occurance_list.clone().iter())
    .for_each(|(polynomial,variable)|{

        let does_exist = witness_values.contains_key(variable);
        match does_exist {
            true =>{
                let var_value = witness_values.get(variable).unwrap();
                let var_value_str = var_value.as_str().unwrap();
                let var_value_u64 = var_value_str.parse::<u64>().unwrap();
                final_polynomial = &final_polynomial + &polynomial.mul(&DensePolynomial::from_coefficients_vec(vec![ScalarField::from(var_value_u64),ScalarField::zero()]))
            }
            false => panic!("Variable: {:?} not found in the witness file",&variable)
        }   
    }); 
    
    final_polynomial
}

fn extract_g1_element(element:ProjectiveConfigType)->Projective<Config>{
    match element {
        ProjectiveConfigType::GOne(ref elem) => elem.clone(),
        _ => panic!("Expected GOne element but found a different variant."),
    }
}
fn extract_g2_element(element:ProjectiveConfigType)->Projective<Config2>{
    match element {
        ProjectiveConfigType::GTwo(ref elem) => elem.clone(),
        _ => panic!("Expected GTwo element but found a different variant."),
    }
}

fn compute_encrypted_polynomial_evaluation(g_operand_poly_eval:Vec<ProjectiveConfigType>,occurance_list:Vec<String>,g_t_eval_delta:Projective<Config>,witness_values:HashMap<String,Value>)->Projective<Config>{

    let mut g_lop_eval:G = G::zero();
    g_operand_poly_eval.clone().iter()
    .zip(occurance_list.clone().iter())
    .for_each(|(_g_lop_eval_part,variable)|{
        let g_lop_eval_part = extract_g1_element(*_g_lop_eval_part);

        let does_exist = witness_values.contains_key(variable);
        match does_exist {
            true =>{
                let var_value = witness_values.get(variable).unwrap();
                let var_value_str = var_value.as_str().unwrap();
                let var_value_u64 = var_value_str.parse::<u64>().unwrap();
                g_lop_eval = g_lop_eval +  g_lop_eval_part.mul(ScalarField::from(var_value_u64));
            }
            false => panic!("Variable: {:?} not found in the witness file",&variable)
        }   
    }); 

    g_lop_eval = g_lop_eval + g_t_eval_delta; //Final  part
    g_lop_eval

}

fn compute_encrypted_polynomial_evaluation_g2(g_operand_poly_eval:Vec<ProjectiveConfigType>,occurance_list:Vec<String>,g_t_eval_delta:Projective<Config2>,witness_values:HashMap<String,Value>)->Projective<Config2>{

    let mut g_lop_eval:G2 = G2::zero();
    g_operand_poly_eval.clone().iter()
    .zip(occurance_list.clone().iter())
    .for_each(|(_g_lop_eval_part,variable)|{
        let g_lop_eval_part = extract_g2_element(*_g_lop_eval_part);

        let does_exist = witness_values.contains_key(variable);
        match does_exist {
            true =>{
                let var_value = witness_values.get(variable).unwrap();
                let var_value_str = var_value.as_str().unwrap();
                let var_value_u64 = var_value_str.parse::<u64>().unwrap();
                g_lop_eval = g_lop_eval +  g_lop_eval_part.mul(ScalarField::from(var_value_u64));
            }
            false => panic!("Variable: {:?} not found in the witness file",&variable)
        }   
    }); 

    g_lop_eval = g_lop_eval + g_t_eval_delta; //Final  part
    g_lop_eval

}


fn main(){

    //Read proving key
    let proving_key = load_key_from_file("proving_key.bin",30).unwrap();

    //Read verification key
    let verification_key = load_key_from_file("verification_key.bin",13).unwrap();
    
    //Read witness values
    let witness_values = load_witness_values().unwrap();

    let parsed_operations = parse_circuit("./src/prover/prover_polynomial.pika");
    println!("Operations: {:?}", parsed_operations);

    let (left_op_points,left_occurance_list) = compute_op_points(parsed_operations.clone(), 0);
    let (right_op_points,right_occurance_list) = compute_op_points(parsed_operations.clone(), 1);
    let (ouput_op_points,output_occurance_list) = compute_op_points(parsed_operations.clone(), 2);

    //Lagrange interpolation
    let (left_operand_polynomial_array,_) = compute_op_polynomial(left_op_points);
    let (right_operand_polynomial_array,_) = compute_op_polynomial(right_op_points);
    let (output_operand_polynomial_array,_) = compute_op_polynomial(ouput_op_points);

    let vanishing_p = compute_vanishing_polynomial(parsed_operations.len());

    for (key, value) in &witness_values {
        println!("{}: {}", key, value);
    }

    //Compute operand polynomial
    let left_operand_polynomial = compute_final_polynomial(witness_values.clone(),left_operand_polynomial_array.clone(),left_occurance_list.clone());
    let right_operand_polynomial = compute_final_polynomial(witness_values.clone(),right_operand_polynomial_array.clone(),right_occurance_list.clone());
    let output_operand_polynomial = compute_final_polynomial(witness_values.clone(),output_operand_polynomial_array.clone(),output_occurance_list.clone());
    
    let polynomial_p = &left_operand_polynomial.mul(&right_operand_polynomial) - &output_operand_polynomial;
    let (polynomial_h_p1,remainder)= DenseOrSparsePolynomial::from(polynomial_p.clone()).divide_with_q_and_r(&DenseOrSparsePolynomial::from(vanishing_p.clone())).unwrap();
    
    println!("Quotient: {:?}",polynomial_h_p1);
    println!("Remainder: {:?}",remainder);
    assert_eq!(DensePolynomial::from_coefficients_vec(vec![ScalarField::zero()]),remainder); //Remainder should be 0 for valid polynomial

    // Compute random deltal,deltar,deltao
    let mut rng = ark_std::test_rng();
    let delta_l:ScalarField = ScalarField::rand(&mut rng);
    let delta_r:ScalarField = ScalarField::rand(&mut rng);
    let delta_o:ScalarField = ScalarField::rand(&mut rng);
    let delta_l_r:ScalarField = delta_l*delta_r;

    let polynomial_h_p2 = &left_operand_polynomial * delta_r + &right_operand_polynomial*delta_l + &vanishing_p *delta_l_r;
    let polynomial_h = polynomial_h_p1 + polynomial_h_p2.sub(&DensePolynomial::from_coefficients_vec(vec![delta_o]));

    //Evaluations
    let gl_left_operand_poly_eval = proving_key[0].clone();
    let gr_right_operand_poly_eval = proving_key[1].clone();
    let go_output_operand_poly_eval = proving_key[2].clone();
    let gl_alpha_left_operand_poly_eval = proving_key[3].clone();
    let gr_alpha_right_operand_poly_eval = proving_key[4].clone();
    let go_alpha_output_operand_poly_eval = proving_key[5].clone();
    let gl_beta_left_operand_poly_eval = proving_key[6].clone();
    let gr_beta_right_operand_poly_eval = proving_key[7].clone();
    let go_beta_output_operand_poly_eval = proving_key[8].clone();
    let g_vanishing_eval = proving_key[9].clone();
    let gr2_vanishing_eval = proving_key[10].clone(); //G2
    let gr2_right_operand_poly_eval = proving_key[11].clone(); //G2
    let gr2_alpha_right_operand_poly_eval = proving_key[12].clone(); //G2
    let g2sk = proving_key[13].clone(); //G2
    

    let gl_t_eval: Projective<Config> = extract_g1_element(g_vanishing_eval[0]); //gl^t(s)
    // let gr_t_eval: Projective<Config> = extract_g1_element(g_vanishing_eval[1]); //gr^t(s)
    let gr2_t_eval = extract_g2_element(gr2_vanishing_eval[0]);//gr2^t(s) //G2
    let go_t_eval: Projective<Config> = extract_g1_element(g_vanishing_eval[2]); //go^t(s)


    let gl_t_eval_deltal = gl_t_eval.mul(delta_l); // gl^t(s)^deltal
    // let gr_t_eval_deltar = gr_t_eval.mul(delta_r); // gr^t(s)^deltar
    let gr2_t_eval_deltar = gr2_t_eval.mul(delta_r);//gr2^t(s)^deltar //G2
    let go_t_eval_deltao = go_t_eval.mul(delta_o); // go^t(s)^deltao


    
    let gl_alphal_t_eval:Projective<Config> = extract_g1_element(g_vanishing_eval[3]); //gl^alphal*t(s)
    // let gr_alphar_t_eval:Projective<Config> = extract_g1_element(g_vanishing_eval[4]); //gr^alphal*t(s)
    let gr2_alphar_t_eval = extract_g2_element(gr2_vanishing_eval[1]);//gr2^t(s) //G2
    let go_alphao_t_eval:Projective<Config> = extract_g1_element(g_vanishing_eval[5]); //go^alphal*t(s)



    let gl_alphal_t_eval_deltal = gl_alphal_t_eval.mul(delta_l); //gl^alphal*t(s)^deltal
    // let gr_alphar_t_eval_deltar = gr_alphar_t_eval.mul(delta_r); //gr^alphar*t(s)^deltar
    let go_alphao_t_eval_deltao = go_alphao_t_eval.mul(delta_o); //go^alphao*t(s)^deltao

    let gr2_alphar_t_eval_deltar = gr2_alphar_t_eval.mul(delta_r);//gr2^t(s) //G2

    
    //Compute gl^LP(s)
    let gl_lop_eval = compute_encrypted_polynomial_evaluation(gl_left_operand_poly_eval,left_occurance_list.clone(),gl_t_eval_deltal,witness_values.clone());
    
    //Compute gl^L'p(s)
    let gl_lop_shifted_eval =  compute_encrypted_polynomial_evaluation(gl_alpha_left_operand_poly_eval.clone(),left_occurance_list.clone(),gl_alphal_t_eval_deltal,witness_values.clone());


    //Compute gr^RP(s)
    let gr_rop_eval = compute_encrypted_polynomial_evaluation_g2(gr2_right_operand_poly_eval,right_occurance_list.clone(),gr2_t_eval_deltar,witness_values.clone());

    //Compute gr^R'p(s)
    let gr_rop_shifted_eval =  compute_encrypted_polynomial_evaluation_g2(gr2_alpha_right_operand_poly_eval.clone(),right_occurance_list.clone(),gr2_alphar_t_eval_deltar,witness_values.clone());

    //Compute go^OP(s)
    let go_oop_eval = compute_encrypted_polynomial_evaluation(go_output_operand_poly_eval,output_occurance_list.clone(),go_t_eval_deltao,witness_values.clone());

    //Compute go^O'p(s)
    let go_oop_shifted_eval = compute_encrypted_polynomial_evaluation(go_alpha_output_operand_poly_eval.clone(),output_occurance_list.clone(),go_alphao_t_eval_deltao,witness_values.clone());

    
    //Fetch values from verification key and test
    let generator_g1 = extract_g1_element(verification_key[0][0].clone());
    let generator_g2: Projective<Config2> = extract_g2_element(verification_key[5][0].clone());
    let g_alphal_g2 = extract_g2_element(verification_key[5][1].clone());
    // let g_alphar_g2 = extract_g2_element(verification_key[5][2].clone());
    let g_alphar_g1 = extract_g1_element(verification_key[4][2].clone());
    let go_t_eval = extract_g1_element(verification_key[4][0].clone());
    let g_alphao_g2 = extract_g2_element(verification_key[5][3].clone());


    //Pairing check
    let left_part_pairing_l = Bn254::pairing(gl_lop_eval, g_alphal_g2);
    let right_part_pairing_l = Bn254::pairing(gl_lop_shifted_eval, generator_g2);

    let left_part_pairing_r = Bn254::pairing(g_alphar_g1,gr_rop_eval);
    let right_part_pairing_r = Bn254::pairing(generator_g1,gr_rop_shifted_eval);

    let left_part_pairing_o = Bn254::pairing(go_oop_eval, g_alphao_g2);
    let right_part_pairing_o = Bn254::pairing(go_oop_shifted_eval, generator_g2);


    assert_eq!(left_part_pairing_l,right_part_pairing_l);
    assert_eq!(left_part_pairing_r,right_part_pairing_r);
    assert_eq!(left_part_pairing_o,right_part_pairing_o);

    //Asserting the same values G1 and G2 elements 
    let g1_gen = G::generator();
    let g2_gen = G2::generator();

    let generator_l = Bn254::pairing(generator_g1,g2_gen);
    let generator_r = Bn254::pairing(g1_gen,generator_g2);

    assert_eq!(generator_l,generator_r);

    //Compute g^h(s) and test 
    let h_coeffs:Vec<ScalarField> = polynomial_h.clone().coeffs; //Linearly stored 
    let mut g2_h = G2::zero(); //g_h

    for (i,coeff) in h_coeffs.iter().enumerate() {
        if i == 0{
            g2_h = g2_h + generator_g2.mul(coeff);
        }else{
            let g_s_k = extract_g2_element(g2sk[i-1]);
            g2_h = g2_h + g_s_k.mul(coeff);
        }
    }

    // Pairing check for valid operation check  e(gl^Lp(s),gr^Rp(s)) === e(go^(t(s)),g^h(s)) * e(go^O(s),g)
    let left_pairing_part = Bn254::pairing(gl_lop_eval, gr_rop_eval);
    let right_pairing_part_1 = Bn254::pairing(go_t_eval, g2_h);
    let right_pairing_part_2 = Bn254::pairing(go_oop_eval, generator_g2);

    let right_pairing_part = right_pairing_part_1 + right_pairing_part_2;

    assert_eq!(left_pairing_part,right_pairing_part);

}