use core::panic;
use std::{vec};
use ark_bn254::g1::Config;
use ark_ec::short_weierstrass::Projective;
use ark_ec::Group;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use ark_bn254::{G1Projective as G, Fr as ScalarField};
use ark_std::{Zero, UniformRand, ops::Mul,ops::Sub};
use ark_poly::{univariate::DensePolynomial, DenseUVPolynomial,Polynomial};
use ark_poly::{univariate::DenseOrSparsePolynomial};
use pikachu::lagrange_interpolation_polynomial;
use ark_bn254::Fr;


fn shield_brack_parser(op: &str) -> (String, String) {
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

fn parse_circuit() -> Vec<[String; 5]> {
    let path = Path::new("circuit.pika");
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
    println!("Unparsed Operations: {:?}", operations);

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
                println!("Val Vec: {:?}", val_vec);

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

fn compute_op_points(parsed_operations: Vec<[String; 5]>, op_type: i32) -> Vec<Vec<[i32; 2]>> {
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

    println!("Occurance list: {:?}",occurance_list);

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

            println!("[X,Y]: [{:?},{:?}]",x,y);
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
    println!("Operand points: {:?}", op_points_list);
    return op_points_list;
}

fn compute_op_polynomial(op_points: Vec<Vec<[i32; 2]>>) ->(Vec<DensePolynomial<Fr>>,DensePolynomial<Fr>) {

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

        println!("X_POINT_LIST: {:?}",x_point_list_u64);
        println!("Y_POINT_LIST: {:?}",y_point_list_u64);


        let pair_point_list: Vec<(Fr, Fr)> = x_point_list_u64.iter()
        .zip(y_point_list_u64.iter())
        .map(|(&x, &y)| (Fr::from(x), Fr::from(y)))
        .collect();


        //Interpolate polynomial from those points
        let c0_polynomial = lagrange_interpolation_polynomial(&pair_point_list);
        println!("Polynomial: {:?}",c0_polynomial);

        polynomial_array.push(c0_polynomial);

    }

    println!("Polynomial in the operand : {:?}",polynomial_array);

    //Compute final polynomial
    for poly in &polynomial_array{
        final_polynomial = &final_polynomial + poly;
    }

    println!("Final polynomial: {:?}",final_polynomial);

    return (polynomial_array,final_polynomial);


}


fn compute_vanishing_polynomial(length:usize) -> DensePolynomial<Fr> {
    
    let mut vanishing_index:u64 = 0;    
    let mut vanishing_list:Vec<[Fr;2]> = Vec::new();
    let mut count = 0;
    // let p = <Fr as PrimeField>::MODULUS;
    // // let p:u64 = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

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

fn main() {
    let parsed_operations = parse_circuit();
    println!("Operations: {:?}", parsed_operations);

    let left_op_points = compute_op_points(parsed_operations.clone(), 0);
    let right_op_points = compute_op_points(parsed_operations.clone(), 1);
    let ouput_op_points = compute_op_points(parsed_operations.clone(), 2);

    //Lagrange interpolation
    let (left_operand_polynomial_array,left_operand_polynomial) = compute_op_polynomial(left_op_points);
    let (right_operand_polynomial_array,right_operand_polynomial) = compute_op_polynomial(right_op_points);
    let (output_operand_polynomial_array,output_operand_polynomial) = compute_op_polynomial(ouput_op_points);


    // --- Test --- Start
    let a = Fr::from(1u64);
    let b = Fr::from(2u64);
    let c = Fr::from(1u64);
    let r1 = Fr::from(12u64);
    let r2 = Fr::from(1u64);

    println!("LEFTOPARRAY: {:?}",left_operand_polynomial_array);
    println!("LEFTOP: {:?}",left_operand_polynomial);

    let final_left_polynomial = left_operand_polynomial.mul(&DensePolynomial::from_coefficients_vec(vec![a,Fr::zero()]));
    let mut final_right_polynomial = DensePolynomial::from_coefficients_vec(vec![Fr::zero()]);
    let mut final_out_polynomial = DensePolynomial::from_coefficients_vec(vec![Fr::zero()]);


    for (i,poly) in right_operand_polynomial_array.iter().enumerate() {
        if i == 0 {
            final_right_polynomial =  final_right_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![b,Fr::zero()]));
        }else if i == 1{
            final_right_polynomial =  final_right_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![c,Fr::zero()]));
        }
    }

    for (i,poly) in output_operand_polynomial_array.iter().enumerate() {
        if i == 0 {
            final_out_polynomial =  final_out_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![r1,Fr::zero()]));
        }else if i == 1{
            final_out_polynomial =  final_out_polynomial + poly.mul(&DensePolynomial::from_coefficients_vec(vec![r2,Fr::zero()]));
        }
    }

    
    let polynomial_p = &final_left_polynomial.mul(&final_right_polynomial) - &final_out_polynomial;
    let vanishing_p = compute_vanishing_polynomial(parsed_operations.len());
    let (qoutient,remainder)= DenseOrSparsePolynomial::from(polynomial_p.clone()).divide_with_q_and_r(&DenseOrSparsePolynomial::from(vanishing_p.clone())).unwrap();

    println!("Quotient: {:?}",qoutient);
    println!("Remainder: {:?}",remainder);

    ///TEST --- Complete

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
    let mut go_beta_output_operand_poly_eval:Vec<Projective<Config>> = Vec::new(); //Porving key

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
        let gi = (g*s)*ScalarField::from(i as u64);
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
}
