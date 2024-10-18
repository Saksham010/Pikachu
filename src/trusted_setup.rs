use std::vec;
use ark_bn254::g1::Config;
use ark_bn254::g2::Config as Config2;
use ark_ec::short_weierstrass::Projective;
use ark_ec::Group;
use ark_ff::{Fp, MontBackend,QuadExtField,Fp2ConfigWrapper};
use std::fs::File;
use ark_bn254::{Fr as ScalarField,FqConfig,Fq2Config, G1Projective as G, G2Projective as G2};
use ark_std::UniformRand;
use ark_poly::Polynomial;
use pikachu::{parse_circuit,compute_op_points,compute_op_polynomial,compute_vanishing_polynomial};
use ark_bn254::Fr;
use ark_serialize::CanonicalSerialize;
use std::io::prelude::*;
use std::io::Result;
use rand::rngs::OsRng; 

//For G1Projective and G2 projective coordinates
#[derive(Debug)]
#[derive(Clone)]
enum ProjectiveCoordinateType{
    C1(Fp<MontBackend<FqConfig, 4>, 4>),
    C2(QuadExtField<Fp2ConfigWrapper<Fq2Config>>)
}

trait ProjectiveCoordinateTypeT{
    fn serialize_uncomp(&self, serialized_data: &mut Vec<u8>);
}

impl ProjectiveCoordinateTypeT for ProjectiveCoordinateType{

    fn serialize_uncomp(&self, serialized_data: &mut Vec<u8>) {
        match self {
            ProjectiveCoordinateType::C1(val)=>{
                val.serialize_uncompressed(serialized_data).unwrap();
            }
            ProjectiveCoordinateType::C2(val)=>{
                val.serialize_uncompressed(serialized_data).unwrap();

            }
        }
    }

}

//For G1Projective and G2 projective elements
#[derive(Debug)]
#[derive(Clone)]
enum ProjectiveConfigType {
    GOne(Projective<Config>),
    GTwo(Projective<Config2>)
}

trait ProjectiveConfigTypeT {
    fn get_coordinates(&self)->(ProjectiveCoordinateType,ProjectiveCoordinateType,ProjectiveCoordinateType);
}

impl ProjectiveConfigTypeT for ProjectiveConfigType {
    fn get_coordinates(&self)->(ProjectiveCoordinateType,ProjectiveCoordinateType,ProjectiveCoordinateType) {

        match self {
            ProjectiveConfigType::GOne(point)=>{
                let x = ProjectiveCoordinateType::C1(point.x);
                let y = ProjectiveCoordinateType::C1(point.y);
                let z = ProjectiveCoordinateType::C1(point.z);
                (x,y,z)

            }
            ProjectiveConfigType::GTwo(point)=>{
                let x = ProjectiveCoordinateType::C2(point.x);
                let y = ProjectiveCoordinateType::C2(point.y);
                let z = ProjectiveCoordinateType::C2(point.z);
                (x,y,z)
            }
        }
    }
    
}

fn save_key_to_file(key:Vec<Vec<ProjectiveConfigType>>,file_name:&str) -> Result<()>{
    let mut file = File::create(file_name).unwrap();
    const DELIMITER:&[u8] = &[0];
    for vector in key {
        for element in vector {
            let(x,y,z) = element.get_coordinates();

            let element_x = x;
            let element_y = y;
            let element_z = z;

            let mut serialized_data_x = Vec::new();
            let mut serialized_data_y = Vec::new();
            let mut serialized_data_z = Vec::new();

            element_x.serialize_uncomp(&mut serialized_data_x);
            element_y.serialize_uncomp(&mut serialized_data_y);
            element_z.serialize_uncomp(&mut serialized_data_z);

            let x_len: Vec<u8> = vec![serialized_data_x.len() as u8];
            let y_len: Vec<u8> = vec![serialized_data_y.len() as u8];
            let z_len: Vec<u8> = vec![serialized_data_z.len() as u8];

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

fn get_s_k(s:Fr,times:usize)->Fr{
    let mut s_final =Fr::from(1u8);
    for _ in 0..times{
        s_final = s_final*s;
    }
    s_final
}

pub fn main() {
    let parsed_operations = parse_circuit("circuit.pika");
    // println!("Operations: {:?}", parsed_operations);

    let (left_op_points,_) = compute_op_points(parsed_operations.clone(), 0);
    let (right_op_points,_) = compute_op_points(parsed_operations.clone(), 1);
    let (ouput_op_points,_) = compute_op_points(parsed_operations.clone(), 2);

    //Lagrange interpolation
    let (left_operand_polynomial_array,_) = compute_op_polynomial(left_op_points);
    let (right_operand_polynomial_array,_) = compute_op_polynomial(right_op_points);
    let (output_operand_polynomial_array,_) = compute_op_polynomial(ouput_op_points);

    let vanishing_p = compute_vanishing_polynomial(parsed_operations.len());

    //Sample random generator
    // let mut rng = ark_std::test_rng();
    let mut rng = OsRng;
    let g = G::generator(); //Generator on the curve
    let g2 = G2::generator(); //Generator on the curve G2projective

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

    let gr2 = g2 * rohr; //G2


    // let mut gsk :Vec<ProjectiveConfigType>= Vec::new(); //Proving key
    let mut g2sk :Vec<ProjectiveConfigType>= Vec::new(); //Proving key (G2)
    let mut gl_left_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key and Verification key
    let mut gr_right_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key and Verification key
    let mut gr2_right_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key and Verification key (G2)
    let mut go_output_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Porving key and Verification key

    let mut gl_alpha_left_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key
    let mut gr_alpha_right_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key
    let mut gr2_alpha_right_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key (G2)
    let mut go_alpha_output_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Porving key

    let mut gl_beta_left_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key
    let mut gr_beta_right_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key
    let mut go_beta_output_operand_poly_eval:Vec<ProjectiveConfigType> = Vec::new(); //Proving key

    let gl_t_eval = gl * t_eval; //Proving key
    let gr_t_eval = gr * t_eval; //Proving key
    let gr2_t_eval = gr2*t_eval;//Proving key(G2)
    let go_t_eval = go * t_eval; //Proving key and Verification key

    let gl_alphal_t_eval = gl * (alphal*t_eval); //Proving key
    let gr_alphar_t_eval = gr * (alphar*t_eval); //Proving key
    let gr2_alphar_t_eval = gr2* (alphar*t_eval); //Proving key (G2)
    let go_alphao_t_eval = go * (alphao*t_eval); //Proving key

    let gl_beta_t_eval = gl * (beta*t_eval); //Proving key
    let gr_beta_t_eval = gr * (beta*t_eval); //Proving key
    let go_beta_t_eval = go * (beta*t_eval); //Proving key

    let g_alphal = g * alphal; //Verification key
    let g_alphar = g * alphar; //Verification key
    let g_alphao = g * alphao; //Verification key
    // let g_gamma = g * gamma; //Verification key
    let g2_gamma = g2 * gamma; //Verification key (G2)
    // let g_beta_gamma = g * (beta*gamma); //Verification key
    let g2_beta_gamma = g2 * (beta*gamma); //Verification key (G2)
    let g_alphal_g2 = g2 * alphal;
    let g_alphar_g2 = g2 * alphar;
    let g_alphao_g2 = g2 * alphao;



    //Compute g^s^k for 0<= k <= no of operations
    for (i,_) in (0..parsed_operations.len()).into_iter().enumerate(){
        let max_index = i+1;
        let s_val = get_s_k(s.clone(),max_index);
        let g2i = g2*s_val;
        g2sk.push(ProjectiveConfigType::GTwo(g2i)); //G2
    }

    //Compute evaluations : gl^li(s) , gl^alphal*li(s) , gl^beta*li(s) 
    for poly in left_operand_polynomial_array {
        let eval:Fr = poly.evaluate(&s);
    
        let gl_alphal_li = gl * (alphal*eval);
        let gl_beta_li = gl * (beta*eval);
        let gl_li = gl*eval;

        gl_alpha_left_operand_poly_eval.push(ProjectiveConfigType::GOne(gl_alphal_li));
        gl_beta_left_operand_poly_eval.push(ProjectiveConfigType::GOne(gl_beta_li));
        gl_left_operand_poly_eval.push(ProjectiveConfigType::GOne(gl_li));
    }

    //Compute evaluations : gr^ri(s) , gr^alphar*ri(s) ,  gr^beta*ri(s) 
    for poly in right_operand_polynomial_array {
        let eval:Fr = poly.evaluate(&s);

        let gr_alphar_ri = gr * (alphar*eval);
        let gr_beta_ri = gr * (beta*eval);
        let gr_ri = gr*eval;

        let gr2_ri = gr2*eval;
        let gr2_alphar_ri = gr2 * (alphar*eval);

        gr2_right_operand_poly_eval.push(ProjectiveConfigType::GTwo(gr2_ri)); //G2
        gr2_alpha_right_operand_poly_eval.push(ProjectiveConfigType::GTwo(gr2_alphar_ri)); //G2

        gr_alpha_right_operand_poly_eval.push(ProjectiveConfigType::GOne(gr_alphar_ri));
        gr_beta_right_operand_poly_eval.push(ProjectiveConfigType::GOne(gr_beta_ri));
        gr_right_operand_poly_eval.push(ProjectiveConfigType::GOne(gr_ri));

    }

    //Compute evaluations : go^oi(s) , go^alphao*oi(s) ,  go^beta*oi(s) 
    for poly in output_operand_polynomial_array {
        let eval:Fr = poly.evaluate(&s);

        let go_alphar_oi = go * (alphao*eval);
        let go_beta_oi = go * (beta*eval);
        let go_oi = go*eval;

        go_alpha_output_operand_poly_eval.push(ProjectiveConfigType::GOne(go_alphar_oi));
        go_beta_output_operand_poly_eval.push(ProjectiveConfigType::GOne(go_beta_oi));
        go_output_operand_poly_eval.push(ProjectiveConfigType::GOne(go_oi));

    }

    // Serialize proving and verification key to bytes and save them in a file
    
    // Provking key part 2
    let pk_2: Vec<ProjectiveConfigType> = vec![
        ProjectiveConfigType::GOne(gl_t_eval),
        ProjectiveConfigType::GOne(gr_t_eval),
        ProjectiveConfigType::GOne(go_t_eval),
        ProjectiveConfigType::GOne(gl_alphal_t_eval),
        ProjectiveConfigType::GOne(gr_alphar_t_eval),
        ProjectiveConfigType::GOne(go_alphao_t_eval),
        ProjectiveConfigType::GOne(gl_beta_t_eval),
        ProjectiveConfigType::GOne(gr_beta_t_eval),
        ProjectiveConfigType::GOne(go_beta_t_eval),
        ProjectiveConfigType::GOne(g_alphal),
        ProjectiveConfigType::GOne(g_alphar),
        ProjectiveConfigType::GOne(g_alphao),
    ];

    //Final proving key
    let proving_key:Vec<Vec<ProjectiveConfigType>> = vec![
        gl_left_operand_poly_eval.clone(),
        gr_right_operand_poly_eval.clone(),
        go_output_operand_poly_eval.clone(),
        gl_alpha_left_operand_poly_eval,
        gr_alpha_right_operand_poly_eval,
        go_alpha_output_operand_poly_eval,
        gl_beta_left_operand_poly_eval,
        gr_beta_right_operand_poly_eval,
        go_beta_output_operand_poly_eval,
        pk_2,
        vec![
            ProjectiveConfigType::GTwo(gr2_t_eval),
            ProjectiveConfigType::GTwo(gr2_alphar_t_eval)
        ],
        gr2_right_operand_poly_eval,
        gr2_alpha_right_operand_poly_eval,
        g2sk.clone(),
    ];


    // Verification key part 2
    let vk_2:Vec<ProjectiveConfigType> = vec![
        ProjectiveConfigType::GOne(go_t_eval),
        ProjectiveConfigType::GOne(g_alphar),
    ];
    

    //Final verification key
    let verification_key:Vec<Vec<ProjectiveConfigType>>= vec![
        vec![ProjectiveConfigType::GOne(g)],
        vk_2,
        vec![
            ProjectiveConfigType::GTwo(g2),
            ProjectiveConfigType::GTwo(g_alphal_g2),
            ProjectiveConfigType::GTwo(g_alphar_g2),
            ProjectiveConfigType::GTwo(g_alphao_g2),
            ProjectiveConfigType::GTwo(g2_gamma),
            ProjectiveConfigType::GTwo(g2_beta_gamma),
        ] //G2 verification key for pairing
    ];

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
