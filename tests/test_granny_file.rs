use opengr2::{GrannyFile, GrannyResolve};
use opengr2::parser::ElementType;

fn test_suzanne(granny_file: &GrannyFile) {
    let art_tool_name = granny_file.find_element("ArtToolInfo.FromArtToolName").unwrap();
    assert_eq!(art_tool_name.name, "FromArtToolName");

    let art_tool_name_element = &art_tool_name.element;
    assert_eq!(*art_tool_name_element, ElementType::String("3D Studio MAX".to_string()));

    let meshes = granny_file.find_element("Meshes").unwrap();
    assert_eq!(meshes.name, "Meshes");

    let meshes_element = &meshes.element;
    if let ElementType::ArrayOfReferences(meshes) = meshes_element {
        assert_eq!(meshes.len(), 1);

        let mesh = &meshes[0];

        let name = mesh.resolve("Name").unwrap();
        assert_eq!(name.element, ElementType::String("default".to_string()));

        let vertex_data = mesh.resolve("PrimaryVertexData.Vertices").unwrap();
        if let ElementType::ArrayOfReferences(vertices) = &vertex_data.element {
            assert_eq!(vertices.len(), 590);
        } else {
            panic!("Unexpected element type of Meshes[0].PrimaryVertexData.Vertices")
        }
    } else {
        panic!("Unexpected element type of meshes")
    }
}

#[test]
fn test_le_7_32bits() {
    let data = include_bytes!("../assets/suzanne_le.gr2");
    let granny_file = GrannyFile::load_from_bytes(data).unwrap();

    test_suzanne(&granny_file)
}

#[test]
fn test_le_7_64bits() {
    let data = include_bytes!("../assets/suzanne_le64.gr2");
    let granny_file = GrannyFile::load_from_bytes(data).unwrap();

    test_suzanne(&granny_file)
}

#[test]
fn test_be_7_32bits() {
    let data = include_bytes!("../assets/suzanne_be.gr2");
    let granny_file = GrannyFile::load_from_bytes(data).unwrap();

    test_suzanne(&granny_file)
}

#[test]
fn test_be_7_64bits() {
    let data = include_bytes!("../assets/suzanne_be64.gr2");
    let granny_file = GrannyFile::load_from_bytes(data).unwrap();

    test_suzanne(&granny_file)
}

#[test]
fn test_textured_external() {
    let data = include_bytes!("../assets/suzanne_textured_external.gr2");
    let granny_file = GrannyFile::load_from_bytes(data).unwrap();

    test_suzanne(&granny_file);

    let materials = granny_file.find_element("Materials").unwrap();
    if let ElementType::ArrayOfReferences(materials) = &materials.element {
        assert_eq!(materials.len(), 2);

        let texture = materials[0].resolve("Texture").unwrap();
        if let ElementType::Reference(texture) = &texture.element {
            assert_eq!(texture.resolve("FromFileName").map(|e| &e.element), Some(&ElementType::String("texture.png".to_string())));
        } else {
            panic!("Texture on Material#0 is from the wrong type")
        }
    } else {
        panic!("Materials is from the wrong type")
    }
}