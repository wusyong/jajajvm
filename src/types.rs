#[derive(Debug)]
pub struct ClassHeader {
    pub magic: u32,
    pub minor_version: u16,
    pub major_version: u16
}

pub struct ClassInfo {
    pub access_flags: u16,
    pub this_calss: u16,
    pub super_class: u16,
}

pub struct MethodInfo {
    pub access_flags: u16,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
}

pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
}

pub struct Code {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code_length: u32,
    pub code: Vec<u8>,
}

pub struct Method {
    pub name: String,
    pub descriptor: String,
    pub code: Code,
}

pub enum ConstantPool {
    Utf8(String), // 1
    Integer { bytes: i32 }, // 3
    Class { starting_index: u16 }, // 7
    MethodOrFieldRef { class_index: u16, name_and_type_index: u16 }, // 9 || 10
    NameAndType { name_index: u16, descriptor_index: u16 }, // 12
}

pub struct ClassFile {
    pub constant_pool: Vec<ConstantPool>,
    pub method: Vec<Method>,
}

pub const i_invokestatic: u8 = 184;
pub const i_invokevirtual: u8 = 182;
pub const i_getstatic: u8 = 178;
pub const i_return: u8 = 177;
pub const i_ireturn: u8 = 172;
pub const i_goto: u8 = 167;
pub const i_if_icmple: u8 = 164;
pub const i_if_icmpgt: u8 = 163;
pub const i_if_icmpge: u8 = 162;
pub const i_if_icmplt: u8 = 161;
pub const i_if_icmpne: u8 = 160;
pub const i_if_icmpeq: u8 = 159;
pub const i_ifle: u8 = 158;
pub const i_ifgt: u8 = 157;
pub const i_ifge: u8 = 156;
pub const i_iflt: u8 = 155;
pub const i_ifne: u8 = 154;
pub const i_ifeq: u8 = 153;
pub const i_iinc: u8 = 132;
pub const i_ineg: u8 = 116;
pub const i_irem: u8 = 112;
pub const i_idiv: u8 = 108;
pub const i_imul: u8 = 104;
pub const i_isub: u8 = 100;
pub const i_iadd: u8 = 96;
pub const i_istore_3: u8 = 62;
pub const i_istore_0: u8 = 59;
pub const i_istore: u8 = 54;
pub const i_iload_3: u8 = 29;
pub const i_iload_0: u8 = 26;
pub const i_iload: u8 = 21;
pub const i_ldc: u8 = 18;
pub const i_sipush: u8 = 17;
pub const i_bipush: u8 = 16;
pub const i_iconst_5: u8 = 8;
pub const i_iconst_0: u8 = 3;
pub const i_iconst_m1: u8 = 2;