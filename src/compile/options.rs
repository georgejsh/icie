#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub enum Codegen {
	Debug,
	Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, evscode::Configurable)]
pub enum Standard {
	#[evscode(name = "C++11")]
	Cpp11,
	#[evscode(name = "C++14")]
	Cpp14,
	#[evscode(name = "C++17")]
	Cpp17,
	#[evscode(name = "C++20")]
	Cpp20,
    #[evscode(name = "Java")]
	Java,
}
use crate::util::path::Path;
use crate::compile::STANDARD;
impl Codegen {
	pub const LIST: &'static [Codegen] = &[Codegen::Debug, Codegen::Release];
    
	pub fn flags_clang(self) -> &'static [&'static str] {
		match &self {
			Codegen::Debug => STANDARD.get().get_debug_flags(),
			Codegen::Release => STANDARD.get().get_release_flags(),
		}
	}
}

impl Standard {
	pub fn flag_clang(self) -> &'static [&'static str] {
		match self {
			Standard::Cpp11 => &["-std=c++11"],
			Standard::Cpp14 => &["-std=c++14"],
			Standard::Cpp17 => &["-std=c++17"],
			Standard::Cpp20 => &["-std=c++20"],
            Standard::Java => &["--release","17"],
		}
	}
    pub fn is_cpp(self) -> bool{
        match self {
			Standard::Cpp11  | Standard::Cpp14 | Standard::Cpp17 | Standard::Cpp20 => true,
            Standard::Java => false,
		}
    }
    pub fn get_executable_extension(self) -> &'static str{
        match self {
			Standard::Cpp11  | Standard::Cpp14 | Standard::Cpp17 | Standard::Cpp20 => "exe",
            Standard::Java => "class",
		}
    }
    pub fn  execute_command(self,mut new_ext: &str )-> Option<String>{
        match self {
			Standard::Cpp11  | Standard::Cpp14 | Standard::Cpp17 | Standard::Cpp20 => None,
            Standard::Java => {
                if(new_ext.contains(".class")) {
                    let class_name = Path::from_native(new_ext.to_string()).file_stem();
                    //class_name = class_name.get(0..(class_name.len()-6)).unwrap().to_string();                    
                    //args.push(&class_name);
                    new_ext="java";
                    return Some(class_name.to_string());
                }
                else {
                    return None;
                }
            } ,
		}
    }
    pub fn get_extenstions(self) -> &'static [&'static str] {
		match self {
			Standard::Cpp11  | Standard::Cpp14 | Standard::Cpp17 | Standard::Cpp20 => &["cpp", "cc", "cxx"],
            Standard::Java => &["java"],
		}
	}
    pub fn get_debug_flags(self) -> &'static [&'static str] {
		match self {
			Standard::Cpp11  | Standard::Cpp14 | Standard::Cpp17 | Standard::Cpp20 => &["-g", "-O2", "-fno-inline-functions"],
            Standard::Java => &["-g"],
		}
	}
    pub fn get_release_flags(self) -> &'static [&'static str] {
		match self {
			Standard::Cpp11  | Standard::Cpp14 | Standard::Cpp17 | Standard::Cpp20 => &["-Ofast"],
            Standard::Java => &["-nowarn"],
		}
	}
    
    
    
}
