use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter;

use glsl::parser::Parse as _;
use glsl::syntax::*;

// TODO:
// Refactor geez
// Better typechecking, more than mats

pub fn transpile(input: String) -> String {
    // Preprocessor step
    let (glsl, defs) = process_macros(input);

    let stage = ShaderStage::parse(glsl);
    match &stage {
        Err(a) => a.info.clone(),
        _ => {
            let mut s = String::new();
            show_translation_unit(&mut s, &stage.unwrap());
            replace_macros(s, defs)
        }
    }
}

// I'm gonna burn in hell for this
static mut INDENT_LEVEL: usize = 3;
fn add_indent() {
    unsafe {
        INDENT_LEVEL += 1;
    }
}
fn sub_indent() {
    unsafe {
        INDENT_LEVEL -= 1;
    }
}
fn get_indent() -> String {
    unsafe { iter::repeat("    ").take(INDENT_LEVEL).collect::<String>() }
}

#[derive(Clone, Debug)]
enum TypeKind {
    Matrix(usize, usize),
    Vector(usize),
    Scalar,
    Struct(String),
}

// And even more for this
static mut SYM_TABLE: Vec<HashMap<String, TypeKind>> = Vec::new();
fn add_sym(name: String, ty: TypeKind) {
    unsafe {
        SYM_TABLE.last_mut().unwrap().insert(name, ty);
    }
}
fn push_sym() {
    unsafe {
        SYM_TABLE.push(HashMap::new());
    }
}
fn pop_sym() {
    unsafe {
        SYM_TABLE.pop();
    }
}
fn lookup_sym(name: &str) -> Option<TypeKind> {
    unsafe {
        SYM_TABLE
            .last()
            .and_then(|x| x.get(name))
            .or(SYM_TABLE.first().and_then(|x| x.get(name)))
            .map(|x| x.clone())
    }
}

// ---- Transpilation code ----
// Need this hack in an attempt to support the preprocessor since GLSL
// crate doesn't really support it.
pub fn process_macros(s: String) -> (String, HashMap<usize, String>) {
    let mut buff = String::new();
    let mut defs: HashMap<usize, String> = HashMap::new();

    push_sym();
    for (i, line) in s.lines().enumerate() {
        if line.trim_start().starts_with("#") {
            // Marker declaration
            buff.push_str(format!("float __LINE{}__;\n", i).as_str());
            let mut rep = String::new();
            show_preprocessor(&mut rep, &Preprocessor::parse(line.trim_start()).unwrap());
            defs.insert(i, rep);
        } else {
            buff.push_str(line);
            buff.push_str("\n");
        }
    }

    (buff, defs)
}

// Put back macros
pub fn replace_macros(s: String, defs: HashMap<usize, String>) -> String {
    let mut buff = String::new();

    for line in s.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("float __LINE") || trimmed.starts_with("static float __LINE") {
            let skip = if trimmed.starts_with("static") {
                19
            } else {
                12
            };
            let num: usize = trimmed
                .chars()
                .skip(skip)
                .take_while(|a| a.is_numeric())
                .collect::<String>()
                .parse()
                .unwrap();

            // TODO: Keep preceeding whitespace
            if let Some(rep) = defs.get(&num) {
                buff.push_str(rep.as_str().trim_start());
            }
        } else {
            buff.push_str(line);
            buff.push_str("\n");
        }
    }

    buff
}

fn translate_glsl_id(s: &str) -> &str {
    match s {
        // Vector types
        "bvec2" => "bool2",
        "bvec3" => "bool3",
        "bvec4" => "bool4",
        "ivec2" => "int2",
        "ivec3" => "int3",
        "ivec4" => "int4",
        "uvec2" => "uint2",
        "uvec3" => "uint3",
        "uvec4" => "uint4",
        "dvec2" => "double2",
        "dvec3" => "double3",
        "dvec4" => "double4",
        "vec2" => "float2",
        "vec3" => "float3",
        "vec4" => "float4",

        //Matrix types
        "mat2" => "float2x2",
        "mat3" => "float3x3",
        "mat4" => "float4x4",
        "mat2x2" => "float2x2",
        "mat2x3" => "float2x3",
        "mat2x4" => "float2x4",
        "mat3x2" => "float3x2",
        "mat3x3" => "float3x3",
        "mat3x4" => "float3x4",
        "mat4x2" => "float4x2",
        "mat4x3" => "float4x3",
        "mat4x4" => "float4x4",

        // Builtins
        "mix" => "lerp",
        "fract" => "frac",
        "texture" => "tex2D",
        "tex2DLod" => "tex2Dlod",
        "refrac" => "refract",
        "mod" => "glsl_mod",
        "atan" => "atan2",
        "floatBitsToInt" => "asint",
        "intBitsToFloat" => "asfloat",
        "dFdx" | "dFdxCoarse" => "ddx",
        "dFdy" | "dFdyCoarse" => "ddy",
        "dFdxFine" => "ddx_fine",
        "dFdyFine" => "ddy_fine",
        "inversesqrt" => "rsqrt",

        a => a,
    }
}

fn get_function_ret_type<'a>(s: &str, args: Vec<Option<TypeKind>>) -> Option<TypeKind> {
    match translate_glsl_id(s) {
        // Vector types
        "bool2" => Some(TypeKind::Vector(2)),
        "bool3" => Some(TypeKind::Vector(3)),
        "bool4" => Some(TypeKind::Vector(4)),
        "int2" => Some(TypeKind::Vector(2)),
        "int3" => Some(TypeKind::Vector(3)),
        "int4" => Some(TypeKind::Vector(4)),
        "uint2" => Some(TypeKind::Vector(2)),
        "uint3" => Some(TypeKind::Vector(3)),
        "uint4" => Some(TypeKind::Vector(4)),
        "double2" => Some(TypeKind::Vector(2)),
        "double3" => Some(TypeKind::Vector(3)),
        "double4" => Some(TypeKind::Vector(4)),
        "float2" => Some(TypeKind::Vector(2)),
        "float3" => Some(TypeKind::Vector(3)),
        "float4" => Some(TypeKind::Vector(4)),

        //Matrix types
        "float2x2" => Some(TypeKind::Matrix(2, 2)),
        "float3x3" => Some(TypeKind::Matrix(3, 3)),
        "float4x4" => Some(TypeKind::Matrix(4, 4)),
        "float2x3" => Some(TypeKind::Matrix(2, 3)),
        "float2x4" => Some(TypeKind::Matrix(2, 4)),
        "float3x2" => Some(TypeKind::Matrix(3, 2)),
        "float3x4" => Some(TypeKind::Matrix(3, 4)),
        "float4x2" => Some(TypeKind::Matrix(4, 2)),
        "float4x3" => Some(TypeKind::Matrix(4, 3)),

        // Builtins
        "lerp" => args[0].clone(),
        "frac" => args[0].clone(),
        "tex2D" => Some(TypeKind::Vector(4)),
        "tex2Dlod" => Some(TypeKind::Vector(4)),
        "refract" => args[0].clone(),
        "glsl_mod" => args[0].clone(),
        "atan2" => args[0].clone(),
        "asint" => args[0].clone(),
        "asfloat" => Some(TypeKind::Scalar),
        "ddx" => args[0].clone(),
        "ddy" => args[0].clone(),
        "ddx_fine" => args[0].clone(),
        "ddy_fine" => args[0].clone(),
        "rsqrt" => args[0].clone(),

        _ => lookup_sym(s),
    }
}

fn get_expr_type(e: &Expr) -> Option<TypeKind> {
    match e {
        Expr::Variable(ref i) => lookup_sym(i.as_str()),
        Expr::IntConst(ref _x) => Some(TypeKind::Scalar),
        Expr::UIntConst(ref _x) => Some(TypeKind::Scalar),
        Expr::BoolConst(ref _x) => Some(TypeKind::Scalar),
        Expr::FloatConst(ref _x) => Some(TypeKind::Scalar),
        Expr::DoubleConst(ref _x) => Some(TypeKind::Scalar),
        Expr::Unary(ref _op, ref e) => get_expr_type(e),
        Expr::Binary(ref op, ref l, ref r) => {
            let (l, r) = (get_expr_type(l), get_expr_type(r));
            match (l.clone(), op, r.clone()) {
                (Some(_), _, Some(TypeKind::Scalar)) => l, // anything op scalar = scalar
                (Some(TypeKind::Scalar), _, Some(_)) => r, // scalar op anything = scalar
                (Some(TypeKind::Vector(_)), _, Some(TypeKind::Vector(_))) => l, // componentwise vector
                (Some(TypeKind::Matrix(_, _)), BinaryOp::Mult, Some(TypeKind::Matrix(_, _))) => {
                    Some(TypeKind::Scalar)
                } // matrix multiplication
                (Some(TypeKind::Matrix(_, _)), _, Some(TypeKind::Matrix(_, _))) => l, // componentwise matrix
                (Some(TypeKind::Vector(_)), BinaryOp::Mult, Some(TypeKind::Matrix(_, _))) => l, // vector matrix mul
                (Some(TypeKind::Matrix(_, _)), BinaryOp::Mult, Some(TypeKind::Vector(_))) => r, // matrix vector mul
                _ => None,
            }
        }
        Expr::Ternary(ref _c, ref s, ref e) => {
            let (l, r) = (get_expr_type(s), get_expr_type(e));
            match (l.clone(), r.clone()) {
                (_, Some(TypeKind::Scalar)) => l,
                (Some(TypeKind::Scalar), _) => r,
                _ => l,
            }
        }
        Expr::Assignment(ref _v, ref _op, ref e) => get_expr_type(e),
        Expr::Bracket(ref _e, ref _a) => None, // TODO: array ignored for now
        Expr::FunCall(FunIdentifier::Identifier(ref id), ref args) => {
            get_function_ret_type(id.0.as_str(), args.iter().map(get_expr_type).collect())
        } // TODO: this can't handle overloads
        Expr::Dot(ref e, ref i) => {
            match get_expr_type(e) {
                Some(TypeKind::Scalar) | Some(TypeKind::Vector(_)) => {
                    // swizzling
                    if i.0.len() == 1 {
                        return Some(TypeKind::Scalar);
                    } else {
                        return Some(TypeKind::Vector(i.0.len()));
                    }
                }
                Some(TypeKind::Matrix(_, _)) => Some(TypeKind::Scalar), // matrix access
                Some(TypeKind::Struct(name)) => lookup_sym(format!("{}.{}", name, i.0).as_str()),
                a => a,
            }
        }
        Expr::PostInc(ref e) => get_expr_type(e),
        Expr::PostDec(ref e) => get_expr_type(e),
        Expr::Comma(ref _a, ref b) => get_expr_type(b),
        _ => None,
    }
}

fn is_matrix(e: &Expr) -> bool {
    match get_expr_type(e) {
        Some(TypeKind::Matrix(_, _)) => true,
        _ => false,
    }
}
fn is_scalar(e: &Expr) -> bool {
    match get_expr_type(e) {
        Some(TypeKind::Scalar) => true,
        _ => false,
    }
}
fn is_struct(e: &Expr) -> bool {
    match get_expr_type(e) {
        Some(TypeKind::Struct(_)) => true,
        _ => false,
    }
}
fn is_vector(e: &Expr) -> bool {
    match get_expr_type(e) {
        Some(TypeKind::Vector(_)) => true,
        _ => false,
    }
}

fn typespec_to_typekind(ty: &TypeSpecifierNonArray) -> Option<TypeKind> {
    match ty {
        TypeSpecifierNonArray::Bool
        | TypeSpecifierNonArray::Int
        | TypeSpecifierNonArray::UInt
        | TypeSpecifierNonArray::Float
        | TypeSpecifierNonArray::Double => Some(TypeKind::Scalar),
        TypeSpecifierNonArray::Vec2
        | TypeSpecifierNonArray::DVec2
        | TypeSpecifierNonArray::BVec2
        | TypeSpecifierNonArray::IVec2
        | TypeSpecifierNonArray::UVec2 => Some(TypeKind::Vector(2)),
        TypeSpecifierNonArray::Vec3
        | TypeSpecifierNonArray::DVec3
        | TypeSpecifierNonArray::BVec3
        | TypeSpecifierNonArray::IVec3
        | TypeSpecifierNonArray::UVec3 => Some(TypeKind::Vector(3)),
        TypeSpecifierNonArray::Vec4
        | TypeSpecifierNonArray::DVec4
        | TypeSpecifierNonArray::BVec4
        | TypeSpecifierNonArray::IVec4
        | TypeSpecifierNonArray::UVec4 => Some(TypeKind::Vector(4)),
        TypeSpecifierNonArray::Mat2 | TypeSpecifierNonArray::DMat2 => Some(TypeKind::Matrix(2, 2)),
        TypeSpecifierNonArray::Mat3 | TypeSpecifierNonArray::DMat3 => Some(TypeKind::Matrix(3, 3)),
        TypeSpecifierNonArray::Mat4 | TypeSpecifierNonArray::DMat4 => Some(TypeKind::Matrix(4, 4)),
        TypeSpecifierNonArray::Mat23 | TypeSpecifierNonArray::DMat23 => {
            Some(TypeKind::Matrix(2, 3))
        }
        TypeSpecifierNonArray::Mat24 | TypeSpecifierNonArray::DMat24 => {
            Some(TypeKind::Matrix(2, 4))
        }
        TypeSpecifierNonArray::Mat32 | TypeSpecifierNonArray::DMat32 => {
            Some(TypeKind::Matrix(3, 2))
        }
        TypeSpecifierNonArray::Mat34 | TypeSpecifierNonArray::DMat34 => {
            Some(TypeKind::Matrix(3, 4))
        }
        TypeSpecifierNonArray::Mat42 | TypeSpecifierNonArray::DMat42 => {
            Some(TypeKind::Matrix(4, 2))
        }
        TypeSpecifierNonArray::Mat43 | TypeSpecifierNonArray::DMat43 => {
            Some(TypeKind::Matrix(4, 3))
        }
        TypeSpecifierNonArray::Struct(ref s) => {
            if let Some(id) = &s.name {
                Some(TypeKind::Struct(id.0.clone()))
            } else {
                None
            }
        }
        TypeSpecifierNonArray::TypeName(ref tn) => Some(TypeKind::Struct(tn.0.clone())),
        _ => None,
    }
}

// Precedence information for transpiling parentheses properly
trait HasPrecedence {
    fn precedence(&self) -> u32;
}

impl HasPrecedence for Expr {
    fn precedence(&self) -> u32 {
        match self {
            // 0 isn't a valid precedence, but we use this to represent atomic expressions
            Self::Variable(_)
            | Self::IntConst(_)
            | Self::UIntConst(_)
            | Self::BoolConst(_)
            | Self::FloatConst(_)
            | Self::DoubleConst(_) => 0,
            // Precedence operator expression is precedence of operator
            Self::Unary(op, _) => op.precedence(),
            Self::Binary(op, _, _) => op.precedence(),
            Self::Ternary(_, _, _) => 15,
            Self::Assignment(_, op, _) => op.precedence(),
            Self::Bracket(_, _)
            | Self::FunCall(_, _)
            | Self::Dot(_, _)
            | Self::PostInc(_)
            | Self::PostDec(_) => 2,
            Self::Comma(_, _) => 17,
        }
    }
}

impl HasPrecedence for UnaryOp {
    fn precedence(&self) -> u32 {
        3
    }
}

impl HasPrecedence for BinaryOp {
    fn precedence(&self) -> u32 {
        match self {
            Self::Mult | Self::Div | Self::Mod => 4,
            Self::Add | Self::Sub => 5,
            Self::LShift | Self::RShift => 6,
            Self::LT | Self::GT | Self::LTE | Self::GTE => 7,
            Self::Equal | Self::NonEqual => 8,
            Self::BitAnd => 9,
            Self::BitXor => 10,
            Self::BitOr => 11,
            Self::And => 12,
            Self::Xor => 13,
            Self::Or => 14,
        }
    }
}

impl HasPrecedence for AssignmentOp {
    fn precedence(&self) -> u32 {
        16
    }
}

pub fn show_identifier<F>(f: &mut F, i: &Identifier)
where
    F: Write,
{
    let rep = match i.0.as_str() {
        "iTime" => "_Time.y",
        "iTimeDelta" => "unity_DeltaTime.x",
        "iChannel0" => "_MainTex",
        "iChannel1" => "_SecondTex",
        "iChannel2" => "_ThirdTex",
        "iChannel3" => "_FourthTex",
        "gl_FragCoord" => "i.uv",
        "iMouse" => "_Mouse",
        //iResolution, iFrame, iChannelTime, iChannelResolution, iMouse, iDate, iSampleRate
        a => a,
    };
    let _ = f.write_str(rep);
}

pub fn show_type_name<F>(f: &mut F, t: &TypeName)
where
    F: Write,
{
    let _ = f.write_str(&t.0);
}

pub fn show_type_specifier_non_array<F>(f: &mut F, t: &TypeSpecifierNonArray)
where
    F: Write,
{
    match *t {
        TypeSpecifierNonArray::Void => {
            let _ = f.write_str("void");
        }
        TypeSpecifierNonArray::Bool => {
            let _ = f.write_str("bool");
        }
        TypeSpecifierNonArray::Int => {
            let _ = f.write_str("int");
        }
        TypeSpecifierNonArray::UInt => {
            let _ = f.write_str("uint");
        }
        TypeSpecifierNonArray::Float => {
            let _ = f.write_str("float");
        }
        TypeSpecifierNonArray::Double => {
            let _ = f.write_str("double");
        }
        TypeSpecifierNonArray::Vec2 => {
            let _ = f.write_str("float2");
        }
        TypeSpecifierNonArray::Vec3 => {
            let _ = f.write_str("float3");
        }
        TypeSpecifierNonArray::Vec4 => {
            let _ = f.write_str("float4");
        }
        TypeSpecifierNonArray::DVec2 => {
            let _ = f.write_str("double2");
        }
        TypeSpecifierNonArray::DVec3 => {
            let _ = f.write_str("double3");
        }
        TypeSpecifierNonArray::DVec4 => {
            let _ = f.write_str("double4");
        }
        TypeSpecifierNonArray::BVec2 => {
            let _ = f.write_str("bool2");
        }
        TypeSpecifierNonArray::BVec3 => {
            let _ = f.write_str("bool3");
        }
        TypeSpecifierNonArray::BVec4 => {
            let _ = f.write_str("bool4");
        }
        TypeSpecifierNonArray::IVec2 => {
            let _ = f.write_str("int2");
        }
        TypeSpecifierNonArray::IVec3 => {
            let _ = f.write_str("int3");
        }
        TypeSpecifierNonArray::IVec4 => {
            let _ = f.write_str("int4");
        }
        TypeSpecifierNonArray::UVec2 => {
            let _ = f.write_str("uint2");
        }
        TypeSpecifierNonArray::UVec3 => {
            let _ = f.write_str("uint3");
        }
        TypeSpecifierNonArray::UVec4 => {
            let _ = f.write_str("uint4");
        }
        TypeSpecifierNonArray::Mat2 => {
            let _ = f.write_str("float2x2");
        }
        TypeSpecifierNonArray::Mat3 => {
            let _ = f.write_str("float3x3");
        }
        TypeSpecifierNonArray::Mat4 => {
            let _ = f.write_str("float4x4");
        }
        TypeSpecifierNonArray::Mat23 => {
            let _ = f.write_str("float2x3");
        }
        TypeSpecifierNonArray::Mat24 => {
            let _ = f.write_str("float2x4");
        }
        TypeSpecifierNonArray::Mat32 => {
            let _ = f.write_str("float3x2");
        }
        TypeSpecifierNonArray::Mat34 => {
            let _ = f.write_str("float3x4");
        }
        TypeSpecifierNonArray::Mat42 => {
            let _ = f.write_str("float4x2");
        }
        TypeSpecifierNonArray::Mat43 => {
            let _ = f.write_str("float4x3");
        }
        TypeSpecifierNonArray::DMat2 => {
            let _ = f.write_str("double2x2");
        }
        TypeSpecifierNonArray::DMat3 => {
            let _ = f.write_str("double3x3");
        }
        TypeSpecifierNonArray::DMat4 => {
            let _ = f.write_str("double4x4");
        }
        TypeSpecifierNonArray::DMat23 => {
            let _ = f.write_str("double2x3");
        }
        TypeSpecifierNonArray::DMat24 => {
            let _ = f.write_str("double2x4");
        }
        TypeSpecifierNonArray::DMat32 => {
            let _ = f.write_str("double3x2");
        }
        TypeSpecifierNonArray::DMat34 => {
            let _ = f.write_str("double3x4");
        }
        TypeSpecifierNonArray::DMat42 => {
            let _ = f.write_str("double4x2");
        }
        TypeSpecifierNonArray::DMat43 => {
            let _ = f.write_str("double4x3");
        }
        TypeSpecifierNonArray::Sampler2D => {
            let _ = f.write_str("sampler2D");
        }
        TypeSpecifierNonArray::Struct(ref s) => show_struct_non_declaration(f, s),
        TypeSpecifierNonArray::TypeName(ref tn) => show_type_name(f, tn),
        _ => panic!("Unexpected type: {:?}", *t),
    }
}

pub fn show_type_specifier<F>(f: &mut F, t: &TypeSpecifier)
where
    F: Write,
{
    show_type_specifier_non_array(f, &t.ty);

    if let Some(ref arr_spec) = t.array_specifier {
        show_array_spec(f, arr_spec);
    }
}

pub fn show_fully_specified_type<F>(f: &mut F, t: &FullySpecifiedType)
where
    F: Write,
{
    if let Some(ref qual) = t.qualifier {
        show_type_qualifier(f, &qual);
        let _ = f.write_str(" ");
    }

    show_type_specifier(f, &t.ty);
}

pub fn show_struct_non_declaration<F>(f: &mut F, s: &StructSpecifier)
where
    F: Write,
{
    let _ = f.write_str("struct ");

    if let Some(ref name) = s.name {
        let _ = write!(f, "{} ", name);
    }

    let _ = f.write_str("\n");
    let _ = f.write_str(get_indent().as_str());
    let _ = f.write_str("{\n");

    add_indent();
    for field in &s.fields.0 {
        show_struct_field(f, field);
    }
    sub_indent();

    let _ = f.write_str(get_indent().as_str());
    let _ = f.write_str("}");
}

pub fn show_struct<F>(f: &mut F, s: &StructSpecifier)
where
    F: Write,
{
    show_struct_non_declaration(f, s);
    let _ = f.write_str(";\n");
}

pub fn show_struct_field<F>(f: &mut F, field: &StructFieldSpecifier)
where
    F: Write,
{
    let _ = f.write_str(get_indent().as_str());

    if let Some(ref qual) = field.qualifier {
        show_type_qualifier(f, &qual);
        let _ = f.write_str(" ");
    }

    show_type_specifier(f, &field.ty);
    let _ = f.write_str(" ");

    // there’s at least one identifier
    let mut identifiers = field.identifiers.0.iter();
    let identifier = identifiers.next().unwrap();

    show_arrayed_identifier(f, identifier);

    // write the rest of the identifiers
    for identifier in identifiers {
        let _ = f.write_str(", ");
        show_arrayed_identifier(f, identifier);
    }

    let _ = f.write_str(";\n");
}

pub fn show_array_spec<F>(f: &mut F, a: &ArraySpecifier)
where
    F: Write,
{
    for dimension in &a.dimensions {
        match *dimension {
            ArraySpecifierDimension::Unsized => {
                let _ = f.write_str("[]");
            }
            ArraySpecifierDimension::ExplicitlySized(ref e) => {
                let _ = f.write_str("[");
                show_expr(f, &e);
                let _ = f.write_str("]");
            }
        }
    }
}

pub fn show_arrayed_identifier<F>(f: &mut F, a: &ArrayedIdentifier)
where
    F: Write,
{
    let _ = write!(f, "{}", a.ident);

    if let Some(ref arr_spec) = a.array_spec {
        show_array_spec(f, arr_spec);
    }
}

pub fn show_type_qualifier<F>(f: &mut F, q: &TypeQualifier)
where
    F: Write,
{
    let mut qualifiers = q.qualifiers.0.iter();
    let first = qualifiers.next().unwrap();

    show_type_qualifier_spec(f, first);

    for qual_spec in qualifiers {
        let _ = f.write_str(" ");
        show_type_qualifier_spec(f, qual_spec)
    }
}

pub fn show_type_qualifier_spec<F>(f: &mut F, q: &TypeQualifierSpec)
where
    F: Write,
{
    match *q {
        TypeQualifierSpec::Storage(ref s) => show_storage_qualifier(f, &s),
        TypeQualifierSpec::Layout(ref l) => show_layout_qualifier(f, &l),
        TypeQualifierSpec::Precision(ref p) => show_precision_qualifier(f, &p),
        TypeQualifierSpec::Interpolation(ref i) => show_interpolation_qualifier(f, &i),
        TypeQualifierSpec::Invariant => {
            let _ = f.write_str("invariant");
        }
        TypeQualifierSpec::Precise => {
            let _ = f.write_str("precise");
        }
    }
}

pub fn show_storage_qualifier<F>(f: &mut F, q: &StorageQualifier)
where
    F: Write,
{
    match *q {
        StorageQualifier::Const => {
            let _ = f.write_str("const");
        }
        StorageQualifier::InOut => {
            let _ = f.write_str("inout");
        }
        StorageQualifier::In => {
            let _ = f.write_str("in");
        }
        StorageQualifier::Out => {
            let _ = f.write_str("out");
        }
        StorageQualifier::Centroid => {
            let _ = f.write_str("centroid");
        }
        StorageQualifier::Patch => {
            let _ = f.write_str("patch");
        }
        StorageQualifier::Sample => {
            let _ = f.write_str("sample");
        }
        StorageQualifier::Uniform => {
            let _ = f.write_str("uniform");
        }
        StorageQualifier::Attribute => {
            let _ = f.write_str("attribute");
        }
        StorageQualifier::Varying => {
            let _ = f.write_str("varying");
        }
        StorageQualifier::Buffer => {
            let _ = f.write_str("buffer");
        }
        StorageQualifier::Shared => {
            let _ = f.write_str("shared");
        }
        StorageQualifier::Coherent => {
            let _ = f.write_str("coherent");
        }
        StorageQualifier::Volatile => {
            let _ = f.write_str("volatile");
        }
        StorageQualifier::Restrict => {
            let _ = f.write_str("restrict");
        }
        StorageQualifier::ReadOnly => {
            let _ = f.write_str("readonly");
        }
        StorageQualifier::WriteOnly => {
            let _ = f.write_str("writeonly");
        }
        StorageQualifier::Subroutine(ref n) => show_subroutine(f, &n),
    }
}

pub fn show_subroutine<F>(f: &mut F, types: &Vec<TypeName>)
where
    F: Write,
{
    let _ = f.write_str("subroutine");

    if !types.is_empty() {
        let _ = f.write_str("(");

        let mut types_iter = types.iter();
        let first = types_iter.next().unwrap();

        show_type_name(f, first);

        for type_name in types_iter {
            let _ = f.write_str(", ");
            show_type_name(f, type_name);
        }

        let _ = f.write_str(")");
    }
}

pub fn show_layout_qualifier<F>(f: &mut F, l: &LayoutQualifier)
where
    F: Write,
{
    let mut qualifiers = l.ids.0.iter();
    let first = qualifiers.next().unwrap();

    let _ = f.write_str("layout (");
    show_layout_qualifier_spec(f, first);

    for qual_spec in qualifiers {
        let _ = f.write_str(", ");
        show_layout_qualifier_spec(f, qual_spec);
    }

    let _ = f.write_str(")");
}

pub fn show_layout_qualifier_spec<F>(f: &mut F, l: &LayoutQualifierSpec)
where
    F: Write,
{
    match *l {
        LayoutQualifierSpec::Identifier(ref i, Some(ref e)) => {
            let _ = write!(f, "{} = ", i);
            show_expr(f, &e);
        }
        LayoutQualifierSpec::Identifier(ref i, None) => show_identifier(f, &i),
        LayoutQualifierSpec::Shared => {
            let _ = f.write_str("shared");
        }
    }
}

pub fn show_precision_qualifier<F>(f: &mut F, p: &PrecisionQualifier)
where
    F: Write,
{
    match *p {
        PrecisionQualifier::High => {
            let _ = f.write_str("highp");
        }
        PrecisionQualifier::Medium => {
            let _ = f.write_str("mediump");
        }
        PrecisionQualifier::Low => {
            let _ = f.write_str("low");
        }
    }
}

pub fn show_interpolation_qualifier<F>(f: &mut F, i: &InterpolationQualifier)
where
    F: Write,
{
    match *i {
        InterpolationQualifier::Smooth => {
            let _ = f.write_str("smooth");
        }
        InterpolationQualifier::Flat => {
            let _ = f.write_str("flat");
        }
        InterpolationQualifier::NoPerspective => {
            let _ = f.write_str("noperspective");
        }
    }
}

pub fn show_float<F>(f: &mut F, x: f32)
where
    F: Write,
{
    if x.fract() == 0. {
        let _ = write!(f, "{}.", x);
    } else {
        let _ = write!(f, "{}", x);
    }
}

pub fn show_double<F>(f: &mut F, x: f64)
where
    F: Write,
{
    if x.fract() == 0. {
        let _ = write!(f, "{}.lf", x);
    } else {
        let _ = write!(f, "{}lf", x);
    }
}

pub fn show_expr<F>(f: &mut F, expr: &Expr)
where
    F: Write,
{
    match *expr {
        Expr::Variable(ref i) => show_identifier(f, &i),
        Expr::IntConst(ref x) => {
            let _ = write!(f, "{}", x);
        }
        Expr::UIntConst(ref x) => {
            let _ = write!(f, "{}u", x);
        }
        Expr::BoolConst(ref x) => {
            let _ = write!(f, "{}", x);
        }
        Expr::FloatConst(ref x) => show_float(f, *x),
        Expr::DoubleConst(ref x) => show_double(f, *x),
        Expr::Unary(ref op, ref e) => {
            // Note: all unary ops are right-to-left associative
            show_unary_op(f, &op);

            if e.precedence() > op.precedence() {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            } else if let Expr::Unary(eop, _) = &**e {
                // Prevent double-unary plus/minus turning into inc/dec
                if eop == op && (*eop == UnaryOp::Add || *eop == UnaryOp::Minus) {
                    let _ = f.write_str("(");
                    show_expr(f, &e);
                    let _ = f.write_str(")");
                } else {
                    show_expr(f, &e);
                }
            } else {
                show_expr(f, &e);
            }
        }
        Expr::Binary(ref op, ref l, ref r) => {
            // Handle mat mult
            if *op == BinaryOp::Mult && (is_matrix(l) || is_matrix(r)) {
                let _ = f.write_str("mul(");
                show_expr(f, &l);
                let _ = f.write_str(",");
                show_expr(f, &r);
                let _ = f.write_str(")");
                return;
            }

            // Note: all binary ops are left-to-right associative (<= for left part)
            if l.precedence() <= op.precedence() {
                show_expr(f, &l);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &l);
                let _ = f.write_str(")");
            }

            show_binary_op(f, &op);

            if r.precedence() < op.precedence() {
                show_expr(f, &r);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &r);
                let _ = f.write_str(")");
            }
        }
        Expr::Ternary(ref c, ref s, ref e) => {
            // Note: ternary is right-to-left associative (<= for right part)

            if c.precedence() < expr.precedence() {
                show_expr(f, &c);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &c);
                let _ = f.write_str(")");
            }
            let _ = f.write_str(" ? ");
            show_expr(f, &s);
            let _ = f.write_str(" : ");
            if e.precedence() <= expr.precedence() {
                show_expr(f, &e);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            }
        }
        Expr::Assignment(ref v, ref op, ref e) => {
            // Handle mat mult
            if *op == AssignmentOp::Mult && is_matrix(e) {
                show_expr(f, &v); //TODO: Precedence
                let _ = f.write_str(" = mul(");
                show_expr(f, &e);
                let _ = f.write_str(",");
                show_expr(f, &v);
                let _ = f.write_str(")");
                return;
            }

            // Note: all assignment ops are right-to-left associative
            if v.precedence() < op.precedence() {
                show_expr(f, &v);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &v);
                let _ = f.write_str(")");
            }

            let _ = f.write_str(" ");
            show_assignment_op(f, &op);
            let _ = f.write_str(" ");

            if e.precedence() <= op.precedence() {
                show_expr(f, &e);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            }
        }
        Expr::Bracket(ref e, ref a) => {
            // Note: bracket is left-to-right associative

            if e.precedence() <= expr.precedence() {
                show_expr(f, &e);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            }

            show_array_spec(f, &a);
        }
        Expr::FunCall(ref fun, ref args) => {
            let mut id = String::new();
            show_function_identifier(&mut id, &fun);

            // Handle atan2 overload
            if id == "atan2" && args.len() == 1 {
                id = String::from("atan");
            }

            // Deal with single value vector constructors
            let expected_arity = match id.as_str() {
                "bool2" => 2,
                "bool3" => 3,
                "bool4" => 4,
                "int2" => 2,
                "int3" => 3,
                "int4" => 4,
                "uint2" => 2,
                "uint3" => 3,
                "uint4" => 4,
                "double2" => 2,
                "double3" => 3,
                "double4" => 4,
                "float2" => 2,
                "float3" => 3,
                "float4" => 4,
                "float2x2" => 4,
                "float3x3" => 9,
                "float4x4" => 16,
                "float2x3" => 6,
                "float2x4" => 8,
                "float3x2" => 6,
                "float3x4" => 12,
                "float4x2" => 8,
                "float4x3" => 12,
                _ => -1,
            };
            if expected_arity != -1 && args.len() == 1 {
                let _ = f.write_str("((");
                let _ = f.write_str(id.as_str());
                let _ = f.write_str(")");
                show_expr(f, &args[0]);
                let _ = f.write_str(")");
            } else {
                // Handle wierd tex2D overloads
                let mut args = args.clone();
                if id == "tex2D" && args.len() > 2 {
                    for i in 2..args.len() {
                        args.remove(i);
                    }
                }

                // Normal handling
                let _ = f.write_str(id.as_str());
                let _ = f.write_str("(");

                if !args.is_empty() {
                    let mut args_iter = args.iter();
                    let first = args_iter.next().unwrap();
                    show_expr(f, first);

                    for e in args_iter {
                        let _ = f.write_str(", ");
                        show_expr(f, e);
                    }
                }

                let _ = f.write_str(")");
            }
        }
        Expr::Dot(ref e, ref i) => {
            // Handle stpq swizzles
            let mut i = i.clone();
            if is_vector(e) {
                i.0 =
                    i.0.replace("s", "x")
                        .replace("t", "y")
                        .replace("p", "z")
                        .replace("q", "w");
            }

            // Note: dot is left-to-right associative
            if e.precedence() <= expr.precedence() {
                show_expr(f, &e);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            }
            let _ = f.write_str(".");
            show_identifier(f, &i);
        }
        Expr::PostInc(ref e) => {
            // Note: post-increment is right-to-left associative
            if e.precedence() < expr.precedence() {
                show_expr(f, &e);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            }

            let _ = f.write_str("++");
        }
        Expr::PostDec(ref e) => {
            // Note: post-decrement is right-to-left associative

            if e.precedence() < expr.precedence() {
                show_expr(f, &e);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &e);
                let _ = f.write_str(")");
            }

            let _ = f.write_str("--");
        }
        Expr::Comma(ref a, ref b) => {
            // Note: comma is left-to-right associative

            if a.precedence() <= expr.precedence() {
                show_expr(f, &a);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &a);
                let _ = f.write_str(")");
            }

            let _ = f.write_str(", ");

            if b.precedence() < expr.precedence() {
                show_expr(f, &b);
            } else {
                let _ = f.write_str("(");
                show_expr(f, &b);
                let _ = f.write_str(")");
            }
        }
    }
}

pub fn show_path<F>(f: &mut F, path: &Path)
where
    F: Write,
{
    match path {
        Path::Absolute(s) => {
            let _ = write!(f, "<{}>", s);
        }
        Path::Relative(s) => {
            let _ = write!(f, "\"{}\"", s);
        }
    }
}

pub fn show_unary_op<F>(f: &mut F, op: &UnaryOp)
where
    F: Write,
{
    match *op {
        UnaryOp::Inc => {
            let _ = f.write_str("++");
        }
        UnaryOp::Dec => {
            let _ = f.write_str("--");
        }
        UnaryOp::Add => {
            let _ = f.write_str("+");
        }
        UnaryOp::Minus => {
            let _ = f.write_str("-");
        }
        UnaryOp::Not => {
            let _ = f.write_str("!");
        }
        UnaryOp::Complement => {
            let _ = f.write_str("~");
        }
    }
}

pub fn show_binary_op<F>(f: &mut F, op: &BinaryOp)
where
    F: Write,
{
    match *op {
        BinaryOp::Or => {
            let _ = f.write_str("||");
        }
        BinaryOp::Xor => {
            let _ = f.write_str("^^");
        }
        BinaryOp::And => {
            let _ = f.write_str("&&");
        }
        BinaryOp::BitOr => {
            let _ = f.write_str("|");
        }
        BinaryOp::BitXor => {
            let _ = f.write_str("^");
        }
        BinaryOp::BitAnd => {
            let _ = f.write_str("&");
        }
        BinaryOp::Equal => {
            let _ = f.write_str("==");
        }
        BinaryOp::NonEqual => {
            let _ = f.write_str("!=");
        }
        BinaryOp::LT => {
            let _ = f.write_str("<");
        }
        BinaryOp::GT => {
            let _ = f.write_str(">");
        }
        BinaryOp::LTE => {
            let _ = f.write_str("<=");
        }
        BinaryOp::GTE => {
            let _ = f.write_str(">=");
        }
        BinaryOp::LShift => {
            let _ = f.write_str("<<");
        }
        BinaryOp::RShift => {
            let _ = f.write_str(">>");
        }
        BinaryOp::Add => {
            let _ = f.write_str("+");
        }
        BinaryOp::Sub => {
            let _ = f.write_str("-");
        }
        BinaryOp::Mult => {
            let _ = f.write_str("*");
        }
        BinaryOp::Div => {
            let _ = f.write_str("/");
        }
        BinaryOp::Mod => {
            let _ = f.write_str("%");
        }
    }
}

pub fn show_assignment_op<F>(f: &mut F, op: &AssignmentOp)
where
    F: Write,
{
    match *op {
        AssignmentOp::Equal => {
            let _ = f.write_str("=");
        }
        AssignmentOp::Mult => {
            let _ = f.write_str("*=");
        }
        AssignmentOp::Div => {
            let _ = f.write_str("/=");
        }
        AssignmentOp::Mod => {
            let _ = f.write_str("%=");
        }
        AssignmentOp::Add => {
            let _ = f.write_str("+=");
        }
        AssignmentOp::Sub => {
            let _ = f.write_str("-=");
        }
        AssignmentOp::LShift => {
            let _ = f.write_str("<<=");
        }
        AssignmentOp::RShift => {
            let _ = f.write_str(">>=");
        }
        AssignmentOp::And => {
            let _ = f.write_str("&=");
        }
        AssignmentOp::Xor => {
            let _ = f.write_str("^=");
        }
        AssignmentOp::Or => {
            let _ = f.write_str("|=");
        }
    }
}

pub fn show_function_identifier<F>(f: &mut F, i: &FunIdentifier)
where
    F: Write,
{
    match *i {
        FunIdentifier::Expr(ref e) => show_expr(f, &*e),
        FunIdentifier::Identifier(ref n) => {
            let id = n.0.as_str();
            let _ = f.write_str(translate_glsl_id(id));
        }
    }
}

pub fn show_declaration<F>(f: &mut F, d: &Declaration, newline: bool, global: bool)
where
    F: Write,
{
    match *d {
        Declaration::FunctionPrototype(ref proto) => {
            show_function_prototype(f, &proto);
            let _ = f.write_str(";");
        }
        Declaration::InitDeclaratorList(ref list) => {
            let invalid_static = match list.head.ty.ty.ty {
                TypeSpecifierNonArray::Struct(_) => true,
                TypeSpecifierNonArray::Void => true,
                _ => false,
            };
            if global && !invalid_static {
                let _ = f.write_str("static ");
            }
            show_init_declarator_list(f, &list);
            let _ = f.write_str(";");
        }
        Declaration::Precision(ref qual, ref ty) => {
            /*show_precision_qualifier(f, &qual);
            show_type_specifier(f, &ty);
            let _ = f.write_str(";");*/
        }
        Declaration::Block(ref block) => {
            show_block(f, &block);
            let _ = f.write_str(";");
        }
        Declaration::Global(ref qual, ref identifiers) => {
            show_type_qualifier(f, &qual);

            if !identifiers.is_empty() {
                let mut iter = identifiers.iter();
                let first = iter.next().unwrap();
                show_identifier(f, first);

                for identifier in iter {
                    let _ = write!(f, ", {}", identifier);
                }
            }

            let _ = f.write_str(";");
        }
    }

    if newline {
        let _ = f.write_str("\n");
    }
}

pub fn show_function_prototype<F>(f: &mut F, fp: &FunctionPrototype)
where
    F: Write,
{
    // Add function prototypes to matrix lookup tab
    if let Some(tk) = typespec_to_typekind(&fp.ty.ty.ty) {
        add_sym(fp.name.0.clone(), tk)
    }

    show_fully_specified_type(f, &fp.ty);

    let _ = f.write_str(" ");
    show_identifier(f, &fp.name);

    let _ = f.write_str("(");

    if !fp.parameters.is_empty() {
        let mut iter = fp.parameters.iter();
        let first = iter.next().unwrap();
        show_function_parameter_declaration(f, first);

        for param in iter {
            let _ = f.write_str(", ");
            show_function_parameter_declaration(f, param);
        }
    }

    let _ = f.write_str(")");
}
pub fn show_function_parameter_declaration<F>(f: &mut F, p: &FunctionParameterDeclaration)
where
    F: Write,
{
    match *p {
        FunctionParameterDeclaration::Named(ref qual, ref fpd) => {
            if let Some(ref q) = *qual {
                show_type_qualifier(f, q);
                let _ = f.write_str(" ");
            }

            show_function_parameter_declarator(f, fpd);
        }
        FunctionParameterDeclaration::Unnamed(ref qual, ref ty) => {
            if let Some(ref q) = *qual {
                show_type_qualifier(f, q);
                let _ = f.write_str(" ");
            }

            show_type_specifier(f, ty);
        }
    }
}

pub fn show_function_parameter_declarator<F>(f: &mut F, p: &FunctionParameterDeclarator)
where
    F: Write,
{
    show_type_specifier(f, &p.ty);
    let _ = f.write_str(" ");
    show_arrayed_identifier(f, &p.ident);
}

pub fn show_init_declarator_list<F>(f: &mut F, i: &InitDeclaratorList)
where
    F: Write,
{
    let add_all_sym = |ty: TypeKind| {
        if let Some(id) = i.head.name.clone() {
            add_sym(id.0, ty.clone());
            for decl in &i.tail {
                add_sym(decl.ident.ident.0.clone(), ty.clone());
            }
        }
    };

    if let Some(tk) = typespec_to_typekind(&i.head.ty.ty.ty) {
        add_all_sym(tk);
    }

    show_single_declaration(f, &i.head);

    for decl in &i.tail {
        let _ = f.write_str(", ");
        show_single_declaration_no_type(f, decl);
    }
}

pub fn show_single_declaration<F>(f: &mut F, d: &SingleDeclaration)
where
    F: Write,
{
    show_fully_specified_type(f, &d.ty);

    if let Some(ref name) = d.name {
        let _ = f.write_str(" ");
        show_identifier(f, name);
    }

    if let Some(ref arr_spec) = d.array_specifier {
        show_array_spec(f, arr_spec);
    }

    if let Some(ref initializer) = d.initializer {
        let _ = f.write_str(" = ");
        show_initializer(f, initializer);
    }
}

pub fn show_single_declaration_no_type<F>(f: &mut F, d: &SingleDeclarationNoType)
where
    F: Write,
{
    show_arrayed_identifier(f, &d.ident);

    if let Some(ref initializer) = d.initializer {
        let _ = f.write_str(" = ");
        show_initializer(f, initializer);
    }
}

pub fn show_initializer<F>(f: &mut F, i: &Initializer)
where
    F: Write,
{
    match *i {
        Initializer::Simple(ref e) => show_expr(f, e),
        Initializer::List(ref list) => {
            let mut iter = list.0.iter();
            let first = iter.next().unwrap();

            let _ = f.write_str("{ ");
            show_initializer(f, first);

            for ini in iter {
                let _ = f.write_str(", ");
                show_initializer(f, ini);
            }

            let _ = f.write_str(" }");
        }
    }
}

pub fn show_block<F>(f: &mut F, b: &Block)
where
    F: Write,
{
    show_type_qualifier(f, &b.qualifier);
    let _ = f.write_str(" ");
    show_identifier(f, &b.name);
    let _ = f.write_str("\n");
    let _ = f.write_str(get_indent().as_str());
    let _ = f.write_str("{");

    for field in &b.fields {
        show_struct_field(f, field);

        let _ = f.write_str("\n");
    }
    let _ = f.write_str("}");

    if let Some(ref ident) = b.identifier {
        show_arrayed_identifier(f, ident);
    }
}

pub fn show_function_definition<F>(f: &mut F, fd: &FunctionDefinition)
where
    F: Write,
{
    // Find parameters that are marked 'out'
    let out_params = &fd
        .prototype
        .parameters
        .iter()
        .filter(|x| match x {
            FunctionParameterDeclaration::Named(Some(ty), _) => {
                ty.qualifiers
                    .0
                    .iter()
                    .filter(|y| match y {
                        TypeQualifierSpec::Storage(StorageQualifier::Out) => true,
                        _ => false,
                    })
                    .count()
                    > 0
            }
            _ => false,
        })
        .collect::<Vec<_>>();

    // Make sure they are initialized
    let mut stmts = fd.statement.clone();
    for p in out_params {
        match p {
            FunctionParameterDeclaration::Named(_, decl) => match decl.ty.ty {
                TypeSpecifierNonArray::Struct(_) | TypeSpecifierNonArray::TypeName(_) => {}
                _ => {
                    let assign = Statement::Simple(Box::new(SimpleStatement::Expression(Some(
                        Expr::Assignment(
                            Box::new(Expr::Variable(decl.ident.ident.clone())),
                            AssignmentOp::Equal,
                            Box::new(Expr::IntConst(0)),
                        ),
                    ))));
                    stmts.statement_list.insert(0, assign);
                }
            },
            _ => unreachable!(),
        }
    }

    // Show function prototype
    show_function_prototype(f, &fd.prototype);
    push_sym();

    // Add parameters to current context
    for param in &fd.prototype.parameters {
        if let FunctionParameterDeclaration::Named(_, decl) = param {
            if let Some(tk) = typespec_to_typekind(&decl.ty.ty) {
                add_sym(decl.ident.ident.0.clone(), tk);
            }
        }
    }

    // Show body
    let _ = f.write_str("\n");
    show_compound_statement(f, &stmts, true);
    pop_sym();
}

pub fn show_compound_statement<F>(f: &mut F, cst: &CompoundStatement, whitespace: bool)
where
    F: Write,
{
    if whitespace {
        let _ = f.write_str(get_indent().as_str());
    }
    let _ = f.write_str("{\n");
    add_indent();

    for st in &cst.statement_list {
        show_statement(f, st, true);
    }

    sub_indent();
    if whitespace {
        let _ = f.write_str(get_indent().as_str());
    }
    let _ = f.write_str("}\n");
}

pub fn show_statement<F>(f: &mut F, st: &Statement, whitespace: bool)
where
    F: Write,
{
    match *st {
        Statement::Compound(ref cst) => show_compound_statement(f, cst, whitespace),
        Statement::Simple(ref sst) => show_simple_statement(f, sst, whitespace),
    }
}

pub fn show_simple_statement<F>(f: &mut F, sst: &SimpleStatement, whitespace: bool)
where
    F: Write,
{
    if whitespace {
        let _ = f.write_str(get_indent().as_str());
    }

    match *sst {
        SimpleStatement::Declaration(ref d) => show_declaration(f, d, true, false),
        SimpleStatement::Expression(ref e) => show_expression_statement(f, e),
        SimpleStatement::Selection(ref s) => show_selection_statement(f, s),
        SimpleStatement::Switch(ref s) => show_switch_statement(f, s),
        SimpleStatement::CaseLabel(ref cl) => show_case_label(f, cl),
        SimpleStatement::Iteration(ref i) => show_iteration_statement(f, i),
        SimpleStatement::Jump(ref j) => show_jump_statement(f, j),
    }
}

pub fn show_expression_statement<F>(f: &mut F, est: &ExprStatement)
where
    F: Write,
{
    if let Some(ref e) = *est {
        show_expr(f, e);
    }

    let _ = f.write_str(";\n");
}

pub fn show_selection_statement<F>(f: &mut F, sst: &SelectionStatement)
where
    F: Write,
{
    let _ = f.write_str("if (");
    show_expr(f, &sst.cond);
    let _ = f.write_str(")\n");
    show_selection_rest_statement(f, &sst.rest);
}

pub fn show_selection_rest_statement<F>(f: &mut F, sst: &SelectionRestStatement)
where
    F: Write,
{
    match *sst {
        SelectionRestStatement::Statement(ref if_st) => {
            let simple = match **if_st {
                Statement::Simple(_) => true,
                _ => false,
            };
            if simple {
                add_indent();
            }
            show_statement(f, if_st, true);
            let _ = f.write_str(get_indent().as_str());
            let _ = f.write_str("\n");
            if simple {
                sub_indent();
            }
        }
        SelectionRestStatement::Else(ref if_st, ref else_st) => {
            show_statement(f, if_st, true);
            let _ = f.write_str(get_indent().as_str());
            let _ = f.write_str("else ");
            match **else_st {
                Statement::Simple(_) => show_statement(f, else_st, false),
                Statement::Compound(ref _st) => {
                    let _ = f.write_str("\n");
                    show_statement(f, else_st, true);
                }
            };
        }
    }
}

pub fn show_switch_statement<F>(f: &mut F, sst: &SwitchStatement)
where
    F: Write,
{
    let _ = f.write_str("switch (");
    show_expr(f, &sst.head);
    let _ = f.write_str(") {\n");

    for st in &sst.body {
        show_statement(f, st, true);
    }

    let _ = f.write_str("}\n");
}

pub fn show_case_label<F>(f: &mut F, cl: &CaseLabel)
where
    F: Write,
{
    match *cl {
        CaseLabel::Case(ref e) => {
            let _ = f.write_str("case ");
            show_expr(f, e);
            let _ = f.write_str(":\n");
        }
        CaseLabel::Def => {
            let _ = f.write_str("default:\n");
        }
    }
}

pub fn show_iteration_statement<F>(f: &mut F, ist: &IterationStatement)
where
    F: Write,
{
    match *ist {
        IterationStatement::While(ref cond, ref body) => {
            let _ = f.write_str("while (");
            show_condition(f, cond);
            let _ = f.write_str(")\n");
            show_statement(f, body, true);
        }
        IterationStatement::DoWhile(ref body, ref cond) => {
            let _ = f.write_str("do ");
            show_statement(f, body, true);
            let _ = f.write_str(" while (");
            show_expr(f, cond);
            let _ = f.write_str(")\n");
        }
        IterationStatement::For(ref init, ref rest, ref body) => {
            let _ = f.write_str("for (");
            show_for_init_statement(f, init);
            show_for_rest_statement(f, rest);
            let _ = f.write_str(")\n");
            show_statement(f, body, true);
        }
    }
}

pub fn show_condition<F>(f: &mut F, c: &Condition)
where
    F: Write,
{
    match *c {
        Condition::Expr(ref e) => show_expr(f, e),
        Condition::Assignment(ref ty, ref name, ref initializer) => {
            show_fully_specified_type(f, ty);
            let _ = f.write_str(" ");
            show_identifier(f, name);
            let _ = f.write_str(" = ");
            show_initializer(f, initializer);
        }
    }
}

pub fn show_for_init_statement<F>(f: &mut F, i: &ForInitStatement)
where
    F: Write,
{
    match *i {
        ForInitStatement::Expression(ref expr) => {
            if let Some(ref e) = *expr {
                show_expr(f, e);
            }
        }
        ForInitStatement::Declaration(ref d) => show_declaration(f, d, false, false),
    }
}

pub fn show_for_rest_statement<F>(f: &mut F, r: &ForRestStatement)
where
    F: Write,
{
    if let Some(ref cond) = r.condition {
        show_condition(f, cond);
    }

    let _ = f.write_str("; ");

    if let Some(ref e) = r.post_expr {
        show_expr(f, e);
    }
}

pub fn show_jump_statement<F>(f: &mut F, j: &JumpStatement)
where
    F: Write,
{
    match *j {
        JumpStatement::Continue => {
            let _ = f.write_str("continue;\n");
        }
        JumpStatement::Break => {
            let _ = f.write_str("break;\n");
        }
        JumpStatement::Discard => {
            let _ = f.write_str("discard;\n");
        }
        JumpStatement::Return(ref e) => {
            let _ = f.write_str("return ");
            if let Some(e) = e {
                show_expr(f, e);
            }
            let _ = f.write_str(";\n");
        }
    }
}

pub fn show_preprocessor<F>(f: &mut F, pp: &Preprocessor)
where
    F: Write,
{
    match *pp {
        Preprocessor::Define(ref pd) => show_preprocessor_define(f, pd),
        Preprocessor::Else => show_preprocessor_else(f),
        Preprocessor::ElseIf(ref pei) => show_preprocessor_elseif(f, pei),
        Preprocessor::EndIf => show_preprocessor_endif(f),
        Preprocessor::Error(ref pe) => show_preprocessor_error(f, pe),
        Preprocessor::If(ref pi) => show_preprocessor_if(f, pi),
        Preprocessor::IfDef(ref pid) => show_preprocessor_ifdef(f, pid),
        Preprocessor::IfNDef(ref pind) => show_preprocessor_ifndef(f, pind),
        Preprocessor::Include(ref pi) => show_preprocessor_include(f, pi),
        Preprocessor::Line(ref pl) => show_preprocessor_line(f, pl),
        Preprocessor::Pragma(ref pp) => show_preprocessor_pragma(f, pp),
        Preprocessor::Undef(ref pu) => show_preprocessor_undef(f, pu),
        Preprocessor::Version(ref pv) => show_preprocessor_version(f, pv),
        Preprocessor::Extension(ref pe) => show_preprocessor_extension(f, pe),
    }
}

pub fn show_preprocessor_define<F>(f: &mut F, pd: &PreprocessorDefine)
where
    F: Write,
{
    let handle_define = |ident: &Identifier, value: &String| {
        let mut res = String::from(value);
        if let Ok(stmt) = Statement::parse(value) {
            res.clear();
            if let Statement::Simple(s) = &stmt {
                if let SimpleStatement::Expression(Some(ref e)) = **s {
                    if let Some(ty) = get_expr_type(e) {
                        add_sym(ident.0.clone(), ty);
                    }
                }
            }
            show_statement(&mut res, &stmt, false);
        } else if let Ok(expr) = Expr::parse(value) {
            res.clear();
            if let Some(ty) = get_expr_type(&expr) {
                add_sym(ident.0.clone(), ty);
            }

            // TODO: This should be recursive
            match expr {
                Expr::Variable(id) => show_expr(
                    &mut res,
                    &Expr::Variable(Identifier(translate_glsl_id(id.0.as_str()).to_owned())),
                ),
                _ => show_expr(&mut res, &expr),
            };
        }
        res
    };

    // TODO: Defines
    match *pd {
        PreprocessorDefine::ObjectLike {
            ref ident,
            ref value,
        } => {
            let res = handle_define(ident, value);

            let _ = write!(f, "#define {} {}\n", ident, res);
        }

        PreprocessorDefine::FunctionLike {
            ref ident,
            ref args,
            ref value,
        } => {
            let _ = write!(f, "#define {}(", ident);

            if !args.is_empty() {
                let _ = write!(f, "{}", &args[0]);

                for arg in &args[1..args.len()] {
                    let _ = write!(f, ", {}", arg);
                }
            }

            let res = handle_define(ident, value);

            let _ = write!(f, ") {}\n", res);
        }
    }
}

pub fn show_preprocessor_else<F>(f: &mut F)
where
    F: Write,
{
    let _ = f.write_str("#else\n");
}

pub fn show_preprocessor_elseif<F>(f: &mut F, pei: &PreprocessorElseIf)
where
    F: Write,
{
    let _ = write!(f, "#elseif {}\n", pei.condition);
}

pub fn show_preprocessor_error<F>(f: &mut F, pe: &PreprocessorError)
where
    F: Write,
{
    let _ = writeln!(f, "#error {}", pe.message);
}

pub fn show_preprocessor_endif<F>(f: &mut F)
where
    F: Write,
{
    let _ = f.write_str("#endif\n");
}

pub fn show_preprocessor_if<F>(f: &mut F, pi: &PreprocessorIf)
where
    F: Write,
{
    let _ = write!(f, "#if {}\n", pi.condition);
}

pub fn show_preprocessor_ifdef<F>(f: &mut F, pid: &PreprocessorIfDef)
where
    F: Write,
{
    let _ = f.write_str("#ifdef ");
    show_identifier(f, &pid.ident);
    let _ = f.write_str("\n");
}

pub fn show_preprocessor_ifndef<F>(f: &mut F, pind: &PreprocessorIfNDef)
where
    F: Write,
{
    let _ = f.write_str("#ifndef ");
    show_identifier(f, &pind.ident);
    let _ = f.write_str("\n");
}

pub fn show_preprocessor_include<F>(f: &mut F, pi: &PreprocessorInclude)
where
    F: Write,
{
    let _ = f.write_str("#include ");
    show_path(f, &pi.path);
    let _ = f.write_str("\n");
}

pub fn show_preprocessor_line<F>(f: &mut F, pl: &PreprocessorLine)
where
    F: Write,
{
    let _ = write!(f, "#line {}", pl.line);
    if let Some(source_string_number) = pl.source_string_number {
        let _ = write!(f, " {}", source_string_number);
    }
    let _ = f.write_str("\n");
}

pub fn show_preprocessor_pragma<F>(f: &mut F, pp: &PreprocessorPragma)
where
    F: Write,
{
    let _ = writeln!(f, "#pragma {}", pp.command);
}

pub fn show_preprocessor_undef<F>(f: &mut F, pud: &PreprocessorUndef)
where
    F: Write,
{
    let _ = f.write_str("#undef ");
    show_identifier(f, &pud.name);
    let _ = f.write_str("\n");
}

pub fn show_preprocessor_version<F>(f: &mut F, pv: &PreprocessorVersion)
where
    F: Write,
{
    let _ = write!(f, "#version {}", pv.version);

    if let Some(ref profile) = pv.profile {
        match *profile {
            PreprocessorVersionProfile::Core => {
                let _ = f.write_str(" core");
            }
            PreprocessorVersionProfile::Compatibility => {
                let _ = f.write_str(" compatibility");
            }
            PreprocessorVersionProfile::ES => {
                let _ = f.write_str(" es");
            }
        }
    }

    let _ = f.write_str("\n");
}

pub fn show_preprocessor_extension<F>(f: &mut F, pe: &PreprocessorExtension)
where
    F: Write,
{
    let _ = f.write_str("#extension ");

    match pe.name {
        PreprocessorExtensionName::All => {
            let _ = f.write_str("all");
        }
        PreprocessorExtensionName::Specific(ref n) => {
            let _ = f.write_str(n);
        }
    }

    if let Some(ref behavior) = pe.behavior {
        match *behavior {
            PreprocessorExtensionBehavior::Require => {
                let _ = f.write_str(" : require");
            }
            PreprocessorExtensionBehavior::Enable => {
                let _ = f.write_str(" : enable");
            }
            PreprocessorExtensionBehavior::Warn => {
                let _ = f.write_str(" : warn");
            }
            PreprocessorExtensionBehavior::Disable => {
                let _ = f.write_str(" : disable");
            }
        }
    }

    let _ = f.write_str("\n");
}

pub fn show_external_declaration<F>(f: &mut F, ed: &ExternalDeclaration)
where
    F: Write,
{
    let _ = f.write_str(get_indent().as_str());
    match *ed {
        ExternalDeclaration::Preprocessor(ref pp) => show_preprocessor(f, pp),
        ExternalDeclaration::FunctionDefinition(ref fd) => {
            show_function_definition(f, fd);
            let _ = f.write_str("\n");
        }
        ExternalDeclaration::Declaration(ref d) => show_declaration(f, d, true, true),
    }
}

pub fn show_translation_unit<F>(f: &mut F, tu: &TranslationUnit)
where
    F: Write,
{
    let _ = f.write_str(
        "Shader \"Converted/Template\"
{
    Properties
    {
        _MainTex (\"iChannel0\", 2D) = \"white\" {}
        _SecondTex (\"iChannel1\", 2D) = \"white\" {}
        _ThirdTex (\"iChannel2\", 2D) = \"white\" {}
        _FourthTex (\"iChannel3\", 2D) = \"white\" {}
        _Mouse (\"Mouse\", Vector) = (0.5, 0.5, 0.5, 0.5)
        [ToggleUI] _GammaCorrect (\"Gamma Correction\", Float) = 1
    }
    SubShader
    {
        Pass
        {
            CGPROGRAM
            #pragma vertex vert
            #pragma fragment frag

            #include \"UnityCG.cginc\"

            struct appdata
            {
                float4 vertex : POSITION;
                float2 uv : TEXCOORD0;
            };

            struct v2f
            {
                float2 uv : TEXCOORD0;
                float4 vertex : SV_POSITION;
            };

            sampler2D _MainTex;   float4 _MainTex_TexelSize;
            sampler2D _SecondTex; float4 _SecondTex_TexelSize;
            sampler2D _ThirdTex;  float4 _ThirdTex_TexelSize;
            sampler2D _FourthTex; float4 _FourthTex_TexelSize;
            float4 _Mouse;
            float _GammaCorrect;

            // GLSL Compatability macros
            #define iFrame (floor(_Time.y / 60))
            #define iResolution float3(1, 1, 1)
            #define glsl_mod(x,y) (((x)-(y)*floor((x)/(y))))
            #define texelFetch(ch, uv, lod) tex2Dlod(ch, float4((uv).xy * ch##_TexelSize.xy + ch##_TexelSize.xy * 0.5, 0, lod))
            #define textureLod(ch, uv, lod) tex2Dlod(ch, float4(uv, 0, lod))

            v2f vert (appdata v)
            {
                v2f o;
                o.vertex = UnityObjectToClipPos(v.vertex);
                o.uv =  v.uv;
                return o;
            }

",
    );

    for ed in &(tu.0).0 {
        match ed {
            ExternalDeclaration::FunctionDefinition(fdef) => {
                if fdef.prototype.name.0.as_str() == "mainImage" {
                    push_sym();

                    let frag = match &fdef.prototype.parameters[0] {
                        FunctionParameterDeclaration::Named(_, name) => name.ident.ident.0.as_str(),
                        _ => panic!(),
                    };
                    let uv = match &fdef.prototype.parameters[1] {
                        FunctionParameterDeclaration::Named(_, name) => name.ident.ident.0.as_str(),
                        _ => panic!(),
                    };

                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_str("float4 frag (v2f i) : SV_Target\n");
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_str("{\n");
                    add_indent();
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_fmt(format_args!("float4 {} = 0;\n", frag));
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_fmt(format_args!("float2 {} = i.uv;\n", uv));
                    for st in &fdef.statement.statement_list {
                        show_statement(f, st, true);
                    }
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_fmt(format_args!(
                        "if (_GammaCorrect) {}.rgb = pow({}.rgb, 2.2);\n",
                        frag, frag
                    ));
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_fmt(format_args!("return {};\n", frag));
                    sub_indent();
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_str("}\n");

                    pop_sym();
                } else {
                    show_external_declaration(f, ed);
                }
            }
            _ => show_external_declaration(f, ed),
        };
    }

    let _ = f.write_str(
        "            ENDCG
        }
    }
}",
    );
}
