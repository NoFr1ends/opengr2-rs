use opengr2::{granny_path, GrannyFile, GrannyPathError};
use opengr2::parser::ElementType;
use opengr2::GrannyResolve;

#[test]
fn test_complex_paths() {
    let data = include_bytes!("../assets/chr.gr2");
    let granny_file = GrannyFile::load_from_bytes(data).unwrap();

    let models = granny_path!(granny_file.elements(), "Models", ElementType::ArrayOfReferences).unwrap();
    for model in models {
        let name = granny_path!(model, "Name", ElementType::String).unwrap();
        println!("Model Name: {}", name);

        let mesh_bindings = granny_path!(model, "MeshBindings", ElementType::ArrayOfReferences).unwrap();
    }
}