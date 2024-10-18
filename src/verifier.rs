use core::panic;
use std::vec;
use ark_bn254::g1::Config;
use ark_bn254::g2::Config as Config2;
use ark_ec::Group; // For the `.mul()` method
use ark_ec::short_weierstrass::Projective;
use std::fs::File;
use std::io::{Cursor, Read};
use ark_bn254::{Bn254, FqConfig,Fq2Config, G1Projective as G, G2Projective as G2};
use ark_serialize::CanonicalDeserialize;
use std::io::Result;
use ark_ff::{Fp,Fp2,Fp2ConfigWrapper,QuadExtField, MontBackend};
use ark_ec::pairing::Pairing;
use base64::{engine::general_purpose, Engine as _}; // Import the Engine trait

//For G1Projective and G2 projective elements
#[derive(Debug)]
#[derive(Clone)]
#[derive(Copy)]
enum ProjectiveConfigType {
    GOne(Projective<Config>),
    GTwo(Projective<Config2>)
}


fn parse_proof(proof:&str) -> Vec<ProjectiveConfigType>{
    let proof_binary:Vec<u8> =  general_purpose::STANDARD.decode(proof).expect("Invalid proof !!");
    let mut cursor = Cursor::new(&proof_binary[..]);
    let mut iteration_no = 0;
    let mut deserialized_proof:Vec<ProjectiveConfigType> = Vec::new();

    //Deserialize proof elements
    while (cursor.position() as usize) < cursor.get_ref().len(){ 
        iteration_no = iteration_no+1;

        //Read the length
        let mut element_len =[0u8];

        cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read x length
        let mut x_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut x_element).expect("Invalid proof !!"); //Read x 

        cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read y length
        let mut y_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut y_element).expect("Invalid proof !!"); //Read y 

        cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read z length
        let mut z_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut z_element).expect("Invalid proof !!"); //Read z

        //Deseralize
        let mut cursorx = Cursor::new(x_element);
        let mut cursory = Cursor::new(y_element);
        let mut cursorz = Cursor::new(z_element);

        if iteration_no >= 7{
            //G2 elements
            let deserialized_x:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursorx).expect("Invalid proof !!");
            let deserialized_y:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursory).expect("Invalid proof !!");
            let deserialized_z:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursorz).expect("Invalid proof !!");
    
            let element:Projective<Config2> = G2::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check
            deserialized_proof.push(ProjectiveConfigType::GTwo(element)); //Push the element
        }else{
            //G1 elements
            let deserialized_x:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorx).expect("Invalid proof !!");
            let deserialized_y:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursory).expect("Invalid proof !!");
            let deserialized_z:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorz).expect("Invalid proof !!");
    
            let element:Projective<Config> = G::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check
            deserialized_proof.push(ProjectiveConfigType::GOne(element)); //Push the element
        }
    }

    deserialized_proof
}

const DELIMITER: &[u8] = &[0]; // Inner delimiter for separating vec of elements
fn load_key_from_file(file_name:&str,g1g2_seperator_index:u8) -> Result<Vec<Vec<ProjectiveConfigType>>>{
    let mut file = File::open(file_name).expect("Invalid proof !!");

    let mut final_key:Vec<Vec<ProjectiveConfigType>> = Vec::new();
    let mut key_vec_inner:Vec<ProjectiveConfigType> = Vec::new();

    //Buffer to load the file
    let mut buffer:Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer).expect("Invalid proof !!");

    // let mut cursor = &buffer[..];
    let mut cursor = Cursor::new(&buffer[..]);

    let mut iteration_no = 0;

    // Deserialize each values
    while (cursor.position() as usize) < cursor.get_ref().len(){ //Ignoring the last 0 delimiter

        //Read the length
        let mut element_len =[0u8];
        cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read x length

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
            cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read again length
        }

        iteration_no = iteration_no+1;

        let mut x_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut x_element).expect("Invalid proof !!"); //Read x 

        cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read y length
        let mut y_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut y_element).expect("Invalid proof !!"); //Read y 

        cursor.read_exact(&mut element_len).expect("Invalid proof !!"); // Read z length
        let mut z_element: Vec<u8> = vec![0u8;element_len[0] as usize];
        cursor.read_exact(&mut z_element).expect("Invalid proof !!"); //Read z

        //Deseralize
        let mut cursorx = Cursor::new(x_element);
        let mut cursory = Cursor::new(y_element);
        let mut cursorz = Cursor::new(z_element);

        if iteration_no >= g1g2_seperator_index {
            //For G2 elements in verification key
            let deserialized_x:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursorx).expect("Invalid proof !!");
            let deserialized_y:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursory).expect("Invalid proof !!");
            let deserialized_z:QuadExtField<Fp2ConfigWrapper<Fq2Config>> = Fp2::deserialize_uncompressed(&mut cursorz).expect("Invalid proof !!");
    
            let element:Projective<Config2> = G2::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check
            key_vec_inner.push(ProjectiveConfigType::GTwo(element)); //Push the element
        }else{
    
            let deserialized_x:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorx).expect("Invalid proof !!");
            let deserialized_y:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursory).expect("Invalid proof !!");
            let deserialized_z:Fp<MontBackend<FqConfig,4>,4> = Fp::deserialize_uncompressed(&mut cursorz).expect("Invalid proof !!");
    
            let element:Projective<Config> = G::new_unchecked(deserialized_x, deserialized_y, deserialized_z); //Note only unchecked returns projective representation, since we construct from already existing group we can ignore the check
            key_vec_inner.push(ProjectiveConfigType::GOne(element)); //Push the element

        }

    }
    Ok(final_key)
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
pub fn main(proof_string:&str){

    let deserialized_proof = parse_proof(proof_string);
    //Read verification key
    let verification_key = load_key_from_file("verification_key.bin",4).expect("Invalid proof !!");

    //Proofs:
    let gl_lop_eval = extract_g1_element(deserialized_proof[0]);
    let gr_rop_eval = extract_g1_element(deserialized_proof[1]);
    let go_oop_eval = extract_g1_element(deserialized_proof[2]);
    let gl_lop_shifted_eval = extract_g1_element(deserialized_proof[3]);
    let go_oop_shifted_eval = extract_g1_element(deserialized_proof[4]);
    let g_z = extract_g1_element(deserialized_proof[5]);
    let gr2_rop_eval = extract_g2_element(deserialized_proof[6]); //G2
    let gr2_rop_shifted_eval = extract_g2_element(deserialized_proof[7]); //G2
    let g2_h = extract_g2_element(deserialized_proof[8]); //G2

    //Verification key:
    let generator_g1 = extract_g1_element(verification_key[0][0].clone());
    let generator_g2: Projective<Config2> = extract_g2_element(verification_key[2][0].clone());
    let g_alphal_g2 = extract_g2_element(verification_key[2][1].clone());
    // let g_alphar_g2 = extract_g2_element(verification_key[3][2].clone());
    let g_alphar_g1 = extract_g1_element(verification_key[1][1].clone());
    let go_t_eval = extract_g1_element(verification_key[1][0].clone());
    let g_alphao_g2 = extract_g2_element(verification_key[2][3].clone());
    let g2_gamma = extract_g2_element(verification_key[2][4].clone());
    let g2_beta_gamma = extract_g2_element(verification_key[2][5].clone());


    //(Pairing check) Variable polynomial restriction check
    let left_part_pairing_l = Bn254::pairing(gl_lop_eval, g_alphal_g2);
    let right_part_pairing_l = Bn254::pairing(gl_lop_shifted_eval, generator_g2);

    let left_part_pairing_r = Bn254::pairing(g_alphar_g1,gr2_rop_eval);
    let right_part_pairing_r = Bn254::pairing(generator_g1,gr2_rop_shifted_eval);

    let left_part_pairing_o = Bn254::pairing(go_oop_eval, g_alphao_g2);
    let right_part_pairing_o = Bn254::pairing(go_oop_shifted_eval, generator_g2);


    assert_eq!(left_part_pairing_l,right_part_pairing_l,"Invalid proof !!"); //Check
    assert_eq!(left_part_pairing_r,right_part_pairing_r,"Invalid proof !!"); //Check
    assert_eq!(left_part_pairing_o,right_part_pairing_o,"Invalid proof !!"); //Check

    //Asserting the same values G1 and G2 elements 
    let g1_gen = G::generator();
    let g2_gen = G2::generator();

    let generator_l = Bn254::pairing(generator_g1,g2_gen);
    let generator_r = Bn254::pairing(g1_gen,generator_g2);

    assert_eq!(generator_l,generator_r,"Invalid proof !!"); //Check

    //(Pairing check) Valid operation check  e(gl^Lp(s),gr^Rp(s)) === e(go^(t(s)),g^h(s)) * e(go^O(s),g)
    let left_pairing_part = Bn254::pairing(gl_lop_eval, gr2_rop_eval);
    let right_pairing_part_1 = Bn254::pairing(go_t_eval, g2_h);
    let right_pairing_part_2 = Bn254::pairing(go_oop_eval, generator_g2);
    let right_pairing_part = right_pairing_part_1 + right_pairing_part_2;

    assert_eq!(left_pairing_part,right_pairing_part,"Invalid proof !!"); //Check

    //Check pairing
    let g_l_r_o = gl_lop_eval + gr_rop_eval + go_oop_eval;
    let variable_pairing_left_part = Bn254::pairing(g_l_r_o, g2_beta_gamma);
    let variable_pairing_right_part = Bn254::pairing(g_z, g2_gamma);

    assert_eq!(variable_pairing_left_part,variable_pairing_right_part,"Invalid proof !!"); //Check

    //Assert the two gr_rop_eval
    let gr_left_part = Bn254::pairing(gr_rop_eval, g2_gen);
    let gr_right_part = Bn254::pairing(g1_gen, gr2_rop_eval);

    assert_eq!(gr_left_part,gr_right_part,"Invalid proof !!"); //Check

    println!("Valid proof !!");
}