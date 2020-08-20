#![allow(non_upper_case_globals)]
use std::fs::File;
use std::io::{BufReader, Read};

mod types;

use types::*;

fn read_1<T: Read>(class_file: &mut T) -> u8 {
    let mut bytes = [0u8; 1];
    class_file.read_exact(&mut bytes).unwrap();
    u8::from_be_bytes(bytes)
}

fn read_2<T: Read>(class_file: &mut T) -> u16 {
    let mut bytes = [0u8; 2];
    class_file.read_exact(&mut bytes).unwrap();
    u16::from_be_bytes(bytes)
}

fn read_4<T: Read>(class_file: &mut T) -> u32 {
    let mut bytes = [0u8; 4];
    class_file.read_exact(&mut bytes).unwrap();
    u32::from_be_bytes(bytes)
}

fn get_constant(constant_pool: &Vec<ConstantPool>, index: u16) -> &ConstantPool {
    &constant_pool[index as usize - 1]
}

fn find_method<'a>(name: &str, desc: &str, methods: &'a Vec<Method>) -> &'a Method {
    for method in methods {
        if method.name == name && method.descriptor == desc {
            return method;
        }
    }

    panic!("Cannot find requested method");
}

fn find_method_from_index(index: u16, class: &ClassFile) -> &Method {
    let name_and_type = get_method_name_and_type(&class.constant_pool, index);
    let name = get_constant(&class.constant_pool, name_and_type.0);
    if let ConstantPool::Utf8(n) = name {
        let descriptor = get_constant(&class.constant_pool, name_and_type.1);
        if let ConstantPool::Utf8(d) = descriptor {
            return find_method(n, d, &class.method);
        }
    }

    panic!("Expected a UTF8");
}

fn get_method_name_and_type(cp: &Vec<ConstantPool>, index: u16) -> (u16, u16) {
    let method = get_constant(cp, index);
    if let &ConstantPool::MethodOrFieldRef { class_index: _, name_and_type_index } = method {
        let name_and_type_constant = get_constant(cp, name_and_type_index);
        if let &ConstantPool::NameAndType { name_index, descriptor_index } = name_and_type_constant {
            return (name_index, descriptor_index);
        }
    }

    panic!("Expected correct MethodRef and NameAndType index");
}

fn get_class_header(class_file: &mut BufReader<File>) -> ClassHeader {
    ClassHeader {
        magic: read_4(class_file),
        minor_version: read_2(class_file),
        major_version: read_2(class_file),
        
    }
}

fn get_constant_pool(class_file: &mut BufReader<File>) -> Vec<ConstantPool> {
    let count = read_2(class_file) as usize - 1;
    let mut cp: Vec<ConstantPool> = Vec::with_capacity(count);

    for _ in 0..count {
        let tag = read_1(class_file);
        match tag {
            1 => {
                let length = read_2(class_file) as usize;
                let mut bytes = vec![0u8; length];
                class_file.read_exact(&mut bytes).unwrap();
                cp.push(ConstantPool::Utf8(String::from_utf8(bytes).unwrap()));
    
            },
            3 => {
                let mut bytes = [0u8; 4];
                class_file.read_exact(&mut bytes).unwrap();
                cp.push(ConstantPool::Integer { bytes: i32::from_be_bytes(bytes) });
            },
            7 => cp.push(ConstantPool::Class { starting_index: read_2(class_file) }),
            9 | 10 => cp.push(ConstantPool::MethodOrFieldRef { class_index: read_2(class_file), name_and_type_index: read_2(class_file)}),
            12 => cp.push(ConstantPool::NameAndType { name_index: read_2(class_file), descriptor_index: read_2(class_file)}),
            x => panic!("Unsupport tag: {}",x),
        }
    }

    cp
}

fn get_class_info(class_file: &mut BufReader<File>) -> ClassInfo {
    let info = ClassInfo {
        access_flags: read_2(class_file),
        this_calss: read_2(class_file),
        super_class: read_2(class_file),
    };
    let interfaces_count = read_2(class_file);
    if interfaces_count != 0 { panic!("This VM does not support interfaces.") }
    let fields_count = read_2(class_file);
    if fields_count != 0 { panic!("This VM does not support fields.") }
    info
}

fn read_method_attributes(class_file: &mut BufReader<File>, acount: u16, cp: &Vec<ConstantPool>) -> Code {
    let mut codes = None;
    let mut found_code = false;
    for _ in 0..acount {
        let ainfo = AttributeInfo {
            attribute_name_index: read_2(class_file),
            attribute_length: read_4(class_file),
        };
        let mut attributes = vec![0u8; ainfo.attribute_length as usize];
        class_file.read_exact(&mut attributes).unwrap();
        let mut attributes = std::io::Cursor::new(attributes);

        let type_constant = get_constant(cp, ainfo.attribute_name_index);
        let type_constant = match type_constant {
            ConstantPool::Utf8(s) => s.clone(),
            _ => panic!("Expected a UTF8"),
        };
        if type_constant == "Code" {
            if found_code { panic!("Duplicate method code"); }
            found_code = true;

            let max_stack = read_2(&mut attributes);
            let max_locals = read_2(&mut attributes);
            let code_length = read_4(&mut attributes);
            let mut code = vec![0u8; code_length as usize];
            attributes.read_exact(&mut code).unwrap();

            codes = Some(Code {
                max_stack,
                max_locals,
                code_length,
                code,
            });
        }
    }

    if !found_code { panic!("Missing method code") }
    codes.unwrap()
}

fn get_methods(class_file: &mut BufReader<File>, cp: &Vec<ConstantPool>) -> Vec<Method> {
    let method_count = read_2(class_file) as usize;
    let mut methods: Vec<Method> = Vec::with_capacity(method_count);

    for _ in 0..method_count {
        let info = MethodInfo {
            access_flags: read_2(class_file),
            name_index: read_2(class_file),
            descriptor_index: read_2(class_file),
            attributes_count: read_2(class_file),
        };

        let name = get_constant(cp, info.name_index);
        let name = match name {
            ConstantPool::Utf8(s) => s.clone(),
            _ => panic!("Expected a UTF8"),
        };
        let descriptor = get_constant(cp, info.descriptor_index);
        let descriptor = match descriptor {
            ConstantPool::Utf8(s) => s.clone(),
            _ => panic!("Expected a UTF8"),
        };

        // FIXME: this VM can only execute static methods, while every class has a constructor method <init>
        if name == "<init>" && (info.access_flags & 0x0008) > 0 { panic!("Only static methods are supported by this VM.") }

        // Read the list of static methods
        let code = read_method_attributes(class_file, info.attributes_count, cp);

        methods.push( Method {
            name,
            descriptor,
            code,
        });
    }

    methods
}

fn get_class(mut class_file: BufReader<File>) -> ClassFile {
    // Read the leading header of the class file
    get_class_header(&mut class_file);

    // Read the constant pool
    let constant_pool =  get_constant_pool(&mut class_file);

    // Read information about the class that was compiled.
    get_class_info(&mut class_file);

    // Read the list of static methods
    let method = get_methods(&mut class_file, &constant_pool);

    ClassFile {
        constant_pool,
        method,
    }
}

fn execute(method: &Method, mut locals: Vec<i32>, class: &ClassFile) -> Option<i32> {
    let code = &method.code;
    let mut op_stack = vec![0i32; code.max_stack as usize];
    let mut op_count = 0;

    // position at the program to be run
    let mut pc = 0;
    let code_buf = &code.code;

    while pc < code.code_length as usize {
        let current = code_buf[pc];
        
        // Reference: https://en.wikipedia.org/wiki/Java_bytecode_instruction_listings
        match current {
            // Return int from method
            i_ireturn => return Some(op_stack[op_count - 1]),
            // Return void from method
            i_return => return None,
            // Invoke a class (static) method
            i_invokestatic => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let index = u16::from_be_bytes([param1, param2]);
                // the method to be called
                let own_method = find_method_from_index(index, class);
                let num_params = own_method.descriptor.len() - 3;
                let mut own_locals = vec![0i32; own_method.code.max_locals as usize];

                for i in (0..num_params).rev() {
                    own_locals[i] = op_stack[op_count - 1];
                    op_count -= 1;
                }

                let exec_res = execute(own_method, own_locals, class);
                if let Some(res) = exec_res {
                    op_stack[op_count] = res;
                    op_count += 1;
                }

                pc += 3;
            },
            // Branch if int comparison with zero succeeds: if equals
            i_ifeq => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let conditional = op_stack[op_count - 1];
                pc += 3;
                if conditional == 0 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 1;
            },
            // Branch if int comparison with zero succeeds: if not equals
            i_ifne => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let conditional = op_stack[op_count - 1];
                pc += 3;
                if conditional != 0 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 1;
            },
            // Branch if int comparison with zero succeeds: if less than 0
            i_iflt => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let conditional = op_stack[op_count - 1];
                pc += 3;
                if conditional < 0 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 1;
            },
            // Branch if int comparison with zero succeeds: if >= 0
            i_ifge => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let conditional = op_stack[op_count - 1];
                pc += 3;
                if conditional >= 0 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 1;
            },
            // Branch if int comparison with zero succeeds: if greater than 0
            i_ifgt => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let conditional = op_stack[op_count - 1];
                pc += 3;
                if conditional > 0 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 1;
            },
            // Branch if int comparison with zero succeeds: if <= 0
            i_ifle => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let conditional = op_stack[op_count - 1];
                pc += 3;
                if conditional <= 0 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 1;
            },
            // Branch if int comparison succeeds: if equals
            i_if_icmpeq => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                pc += 3;
                if op1 == op2 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 2;
            },
            // Branch if int comparison succeeds: if not equals
            i_if_icmpne => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                pc += 3;
                if op1 != op2 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 2;
            },
            // Branch if int comparison succeeds: if less than
            i_if_icmplt => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                pc += 3;
                if op2 < op1 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 2;
            },
            // Branch if int comparison succeeds: if greater than or equal to
            i_if_icmpge => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                pc += 3;
                if op2 >= op1 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 2;
            },
            // Branch if int comparison succeeds: if greater than
            i_if_icmpgt => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                pc += 3;
                if op2 > op1 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 2;
            },
            // Branch if int comparison succeeds: if less than or equal to
            i_if_icmple => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                pc += 3;
                if op2 <= op1 {
                    let res = i16::from_be_bytes([param1, param2]);
                    pc = (pc as i16 + res - 3) as usize;
                }
                op_count -= 2;
            },
            // Branch always
            i_goto => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let res = i16::from_be_bytes([param1, param2]);
                pc = (pc as i16 + res) as usize;
            },
            // Push item from run-time constant pool
            i_ldc => {
                let constant_pool = &class.constant_pool;

                // find the parameter which will be the index from which we retrieve
                // constant in the constant pool.
                let param = code_buf[pc + 1];

                // get the constant
                let info = get_constant(constant_pool, param as u16);
                if let &ConstantPool::Integer{ bytes } = info {
                    op_stack[op_count] = bytes;
                    pc += 2;
                    op_count += 1;
                } else {
                    panic!("Expected Integer");
                }
            },
            // Load int from local variable
            i_iload_0..=i_iload_3 => {
                let param = (current - i_iload_0) as usize;
                let loaded = locals[param];
                op_stack[op_count] = loaded;
                pc += 1;
                op_count += 1;
            },
            i_iload => {
                let param = code_buf[pc + 1] as usize;
                let loaded = locals[param];
                op_stack[op_count] = loaded;
                pc += 2;
                op_count += 1;
            },
            // Store int into local variable
            i_istore => {
                let param = code_buf[pc + 1] as usize;
                let stored = op_stack[op_count - 1];
                locals[param] = stored;
                pc += 2;
                op_count -= 1;
            },
            i_istore_0..=i_istore_3 => {
                let param = (current - i_istore_0) as usize;
                let stored = op_stack[op_count - 1];
                locals[param] = stored;
                pc += 1;
                op_count -= 1;
            },
            // Increment local variable by constant
            i_iinc => {
                let i = code_buf[pc + 1] as usize;
                let b = i8::from_be_bytes([code_buf[pc + 2]]); // signed value
                locals[i] += b as i32;
                pc += 3;
            },
            // Push byte
            i_bipush => {
                let param = i8::from_be_bytes([code_buf[pc + 1]]);
                op_stack[op_count] = param as i32;
                op_count += 1;
                pc += 2;
            },
            // Add int
            i_iadd => {
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                let res = op1.wrapping_add(op2);
                op_stack[op_count - 2] = res;
                op_count -= 1;
                pc += 1;
            },
            // Subtract int
            i_isub => {
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                let res = op2.wrapping_sub(op1);
                op_stack[op_count - 2] = res;
                op_count -= 1;
                pc += 1;
            },
            // Multiply int
            i_imul => {
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                let res = op2.wrapping_mul(op1);
                op_stack[op_count - 2] = res;
                op_count -= 1;
                pc += 1;
            },
            // Divide int
            i_idiv => {
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                let res = op2.wrapping_div(op1);
                op_stack[op_count - 2] = res;
                op_count -= 1;
                pc += 1;
            },
            // Remainder int
            i_irem => {
                let op1 = op_stack[op_count - 1];
                let op2 = op_stack[op_count - 2];
                let res = op2.wrapping_rem(op1);
                op_stack[op_count - 2] = res;
                op_count -= 1;
                pc += 1;
            },
            // Negate int
            i_ineg => {
                let op1 = op_stack[op_count - 1];
                op_stack[op_count - 1] = op1.wrapping_mul(-1);
                pc += 1;
            },
            // Get static field from class
            i_getstatic => pc += 3, // FIXME: unimplemented
            // Invoke instance method; dispatch based on class
            i_invokevirtual => {
                let op = op_stack[op_count - 1];
                // FIXME: the implement is not correct.
                println!("{}", op);
                op_count -= 1;
                pc += 3;
            },
            // Push int constant
            i_iconst_m1..=i_iconst_5 => {
                op_stack[op_count] = current as i32 - i_iconst_0 as i32 ;
                op_count += 1;
                pc += 1;
            },
            // Push short
            i_sipush => {
                let param1 = code_buf[pc + 1];
                let param2 = code_buf[pc + 2];
                let res = i16::from_be_bytes([param1, param2]);
                op_stack[op_count] = res as i32;
                op_count += 1;
                pc += 3;
            }
            _ => unreachable!()
        }
    }

    None
}

fn main() -> std::io::Result<()> {
    // Open class file into buffer reader
    let mut args = std::env::args();
    args.next();
    let file = File::open(args.next().unwrap())?;
    let buf_reader = BufReader::new(file);

    // Parse class file
    let class = get_class(buf_reader);

    // execute the main method if found
    let main_method = find_method("main", "([Ljava/lang/String;)V", &class.method);

    // FIXME: locals[0] contains a reference to String[] args, but right now
    // we lack of the support for java.lang.Object. Leave it uninitialized.
    let locals = vec![0i32; main_method.code.max_locals as usize];
    let result = execute(main_method, locals, &class);

    if let Some(_) = result {
        panic!("main() should return void");
    }

    Ok(())
}
