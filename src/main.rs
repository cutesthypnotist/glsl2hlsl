use std::fmt::Arguments;
use std::fmt::Write;
use std::iter;
use std::ops::Index;

use glsl::parser::Parse as _;
use glsl::syntax::*;
use glsl::visitor::*;

// ---- Setup ----
fn main() {
    let glsl = "/* 
    Combustible Voronoi Layers
	--------------------------

    The effect itself is nothing new or exciting, just some moving 3D Voronoi layering. 
    However, the fire palette might prove useful to some.

*/


// This is my favorite fire palette. It's trimmed down for shader usage, and is based on an 
// article I read at Hugo Elias's site years ago. I'm sure most old people, like me, have 
// visited his site at one time or another:
//
// http://freespace.virgin.net/hugo.elias/models/m_ffire.htm
//
vec3 firePalette(float i){

    float T = 1400. + 1300.*i; // Temperature range (in Kelvin).
    vec3 L = vec3(7.4, 5.6, 4.4); // Red, green, blue wavelengths (in hundreds of nanometers).
    L = pow(L,vec3(5)) * (exp(1.43876719683e5/(T*L)) - 1.);
    return 1. - exp(-5e8/L); 
}

/*
vec3 firePalette(float i){

    float T = 1400. + 1300.*i; // Temperature range (in Kelvin).
    // Hardcode red, green and blue wavelengths (in hundreds of nanometers).
    vec3 L = (exp(vec3(19442.7999572, 25692.271372, 32699.2544734)/T) - 1.);

    return 1. - exp(-vec3(22532.6051122, 90788.296915, 303184.239775)*2.*.5/L); 
}
*/

// Hash function. This particular one probably doesn't disperse things quite as nicely as some 
// of the others around, but it's compact, and seems to work.
//
vec3 hash33(vec3 p){ 
    
    float n = sin(dot(p, vec3(7, 157, 113)));    
    return fract(vec3(2097152, 262144, 32768)*n); 
}

// 3D Voronoi: Obviously, this is just a rehash of IQ's original.
//
float voronoi(vec3 p){

	vec3 b, r, g = floor(p);
	p = fract(p); 
	

	float d = 1.; 
     
    // I've unrolled one of the loops. GPU architecture is a mystery to me, but I'm aware 
    // they're not fond of nesting, branching, etc. My laptop GPU seems to hate everything, 
    // including multiple loops. If it were a person, we wouldn't hang out. 
	for(int j = -1; j <= 1; j++) {
	    for(int i = -1; i <= 1; i++) {
    		
		    b = vec3(i, j, -1);
		    r = b - p + hash33(g+b);
		    d = min(d, dot(r,r));
    		
		    b.z = 0.0;
		    r = b - p + hash33(g+b);
		    d = min(d, dot(r,r));
    		
		    b.z = 1.;
		    r = b - p + hash33(g+b);
		    d = min(d, dot(r,r));
    			
	    }
	}
	
	return d; // Range: [0, 1]
}

// Standard fBm function with some time dialation to give a parallax 
// kind of effect. In other words, the position and time frequencies 
// are changed at different rates from layer to layer.
//
float noiseLayers(in vec3 p) {


    vec3 t = vec3(0., 0., p.z + iTime*1.5);

    const int iter = 5; // Just five layers is enough.
    float tot = 0., sum = 0., amp = 1.; // Total, sum, amplitude.

    for (int i = 0; i < iter; i++) {
        tot += voronoi(p + t) * amp; // Add the layer to the total.
        p *= 2.; // Position multiplied by two.
        t *= 1.5; // Time multiplied by less than two.
        sum += amp; // Sum of amplitudes.
        amp *= .5; // Decrease successive layer amplitude, as normal.
    }
    
    return tot/sum; // Range: [0, 1].
}

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    // Screen coordinates.
	vec2 uv = (fragCoord - iResolution.xy*.5) / iResolution.y;
	
	// Shifting the central position around, just a little, to simulate a 
	// moving camera, albeit a pretty lame one.
	uv += vec2(sin(iTime*.5)*.25, cos(iTime*.5)*.125);
	
    // Constructing the unit ray. 
	vec3 rd = normalize(vec3(uv.x, uv.y, 3.1415926535898/8.));

    // Rotating the ray about the XY plane, to simulate a rolling camera.
	float cs = cos(iTime*.25), si = sin(iTime*.25);
    // Apparently \"r *= rM\" can break in some older browsers.
	rd.xy = rd.xy*mat2(cs, -si, si, cs); 
	
	// Passing a unit ray multiple into the Voronoi layer function, which 
	// is nothing more than an fBm setup with some time dialation.
	float c = noiseLayers(rd*2.);
	
	// Optional: Adding a bit of random noise for a subtle dust effect. 
	c = max(c + dot(hash33(rd)*2. - 1., vec3(.015)), 0.);

    // Coloring:
    
    // Nebula.
    c *= sqrt(c)*1.5; // Contrast.
    vec3 col = firePalette(c); // Palettization.
    //col = mix(col, col.zyx*.1+ c*.9, clamp((1.+rd.x+rd.y)*0.45, 0., 1.)); // Color dispersion.
    col = mix(col, col.zyx*.15 + c*.85, min(pow(dot(rd.xy, rd.xy)*1.2, 1.5), 1.)); // Color dispersion.
    col = pow(col, vec3(1.25)); // Tweaking the contrast a little.
    
    // The fire palette on its own. Perhaps a little too much fire color.
    //c = pow(c*1.33, 1.25);
    //vec3 col =  firePalette(c);
   
    // Black and white, just to keep the art students happy. :)
	//c *= c*1.5;
	//vec3 col = vec3(c);
	
	// Rough gamma correction, and done.
	fragColor = vec4(sqrt(clamp(col, 0., 1.)), 1);
}";

    let stage = ShaderStage::parse(glsl);
    match &stage {
        Err(a) => println!("{}", a.info),
        _ => {
            let mut s = String::new();
            show_translation_unit(&mut s, &stage.unwrap());
            println!("{}", s);
        }
    };
}

// I'm gonna burn in hell for this
static mut INDENT_LEVEL: usize = 3;
fn add_indent() { unsafe { INDENT_LEVEL += 1; } }
fn sub_indent() { unsafe { INDENT_LEVEL -= 1; } }
fn get_indent() -> String {
    unsafe { iter::repeat("    ").take(INDENT_LEVEL).collect::<String>() }
}

// ---- Transpilation code ----
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

    let _ = f.write_str("{\n");

    for field in &s.fields.0 {
        show_struct_field(f, field);
    }

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
            /*match (op, *l.clone(), *r.clone()) {
                // Attempt to handle uv from most shadertoys
                (BinaryOp::Div, Expr::Variable(ll), Expr::Variable(rr)) => {
                    if ll.0 == "fragCoord" && rr.0 == "iResolution" {
                        let _ = f.write_str("i.uv");
                        return;
                    }
                }
                _ => {}
            };*/

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

            // Deal with single value vector constructors
            let expected_arity: isize = match id.as_str() {
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
                _ => -1,
            };
            if expected_arity >= 0 && args.len() == 1 && expected_arity != 1 {
                show_expr(f, &args[0]);
            } else {
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
            let _ = f.write_str(match id {
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
                "mod" => "glsl_mod", // TODO: Handle further up
                "atan" => "atan2",

                a => a,
            });
        }
    }
}

pub fn show_declaration<F>(f: &mut F, d: &Declaration, newline: bool)
where
    F: Write,
{
    match *d {
        Declaration::FunctionPrototype(ref proto) => {
            show_function_prototype(f, &proto);
            let _ = f.write_str(";");
        }
        Declaration::InitDeclaratorList(ref list) => {
            show_init_declarator_list(f, &list);
            let _ = f.write_str(";");
        }
        Declaration::Precision(ref qual, ref ty) => {
            show_precision_qualifier(f, &qual);
            show_type_specifier(f, &ty);
            let _ = f.write_str(";");
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
    let _ = f.write_str(" {");

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
    show_function_prototype(f, &fd.prototype);
    let _ = f.write_str("\n");
    show_compound_statement(f, &fd.statement, true);
}

pub fn show_compound_statement<F>(f: &mut F, cst: &CompoundStatement, whitespace : bool)
where
    F: Write,
{
    if whitespace { let _ = f.write_str(get_indent().as_str()); }
    let _ = f.write_str("{\n");
    add_indent();

    for st in &cst.statement_list {
        show_statement(f, st, true);
    }

    sub_indent();
    if whitespace { let _ = f.write_str(get_indent().as_str()); }
    let _ = f.write_str("}\n");
}

pub fn show_statement<F>(f: &mut F, st: &Statement, whitespace : bool)
where
    F: Write,
{
    match *st {
        Statement::Compound(ref cst) => show_compound_statement(f, cst, whitespace),
        Statement::Simple(ref sst) => show_simple_statement(f, sst, whitespace),
    }
}

pub fn show_simple_statement<F>(f: &mut F, sst: &SimpleStatement, whitespace : bool)
where
    F: Write,
{
    if whitespace { let _ = f.write_str(get_indent().as_str()); }

    match *sst {
        SimpleStatement::Declaration(ref d) => show_declaration(f, d, true),
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
                _ => false
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
                Statement::Compound(ref st) => {
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
        ForInitStatement::Declaration(ref d) => show_declaration(f, d, false),
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
    // TODO: Defines
    match *pd {
        PreprocessorDefine::ObjectLike {
            ref ident,
            ref value,
        } => {
            let _ = write!(f, "#define {} {}\n", ident, value);
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

            let _ = write!(f, ") {}\n", value);
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
        ExternalDeclaration::Declaration(ref d) => show_declaration(f, d, true),
    }
}

pub fn show_translation_unit<F>(f: &mut F, tu: &TranslationUnit)
where
    F: Write,
{
    let _ = f.write_str(
        "
Shader \"Converted/Template\"
{
    Properties
    {
        _MainTex (\"iChannel0\", 2D) = \"white\" {}
        _SecondTex (\"iChannel1\", 2D) = \"white\" {}
        _ThirdTex (\"iChannel2\", 2D) = \"white\" {}
        _FourthTex (\"iChannel3\", 2D) = \"white\" {}
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

            sampler2D _MainTex;
            sampler2D _SecondTex;
            sampler2D _ThirdTex;
            sampler2D _FourthTex;

            #define glsl_mod(x,y) (((x)-(y)*floor((x)/(y)))) 

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
                    let frag = match &fdef.prototype.parameters[0] {
                        FunctionParameterDeclaration::Named(_, name) => name.ident.ident.0.as_str(),
                        _ => panic!()
                    };
                    let uv = match &fdef.prototype.parameters[1] {
                        FunctionParameterDeclaration::Named(_, name) => name.ident.ident.0.as_str(),
                        _ => panic!()
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
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_str("float3 iResolution = 1;\n");
                    for st in &fdef.statement.statement_list {
                        show_statement(f, st, true);
                    }
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_fmt(format_args!("return {};\n", frag));
                    sub_indent();
                    let _ = f.write_str(get_indent().as_str());
                    let _ = f.write_str("}\n");
                } else {
                    show_external_declaration(f, ed);
                }
            }
            _ => show_external_declaration(f, ed),
        };
    }

    let _ = f.write_str(
        "
        ENDCG
        }
    }
}",
    );
}