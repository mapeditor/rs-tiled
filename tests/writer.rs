use std::{fs, path::Path};
use tiled::{
    Loader, PropertyValue,
    writer::{LayerKind, TmxDocument},
};

#[test]
fn edit_and_reopen_map_with_external_and_embedded_tilesets() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.tmx");
    let output = dir.path().join("export");
    fs::create_dir(&output).unwrap();
    fs::write(dir.path().join("external.tsx"), r#"<tileset version="1.10" name="external" tilewidth="8" tileheight="8" tilecount="1" columns="1"><image source="tiles.png" width="8" height="8"/></tileset>"#).unwrap();
    fs::write(&source,r#"<?xml version="1.0"?>
<!-- copyright -->
<map version="1.10" orientation="orthogonal" width="2" height="1" tilewidth="8" tileheight="8" nextobjectid="7">
 <properties><property name="direction" type="int" propertytype="Direction" value="1"/><property name="remove" value="x"/></properties>
 <tileset firstgid="1" source="external.tsx"/>
 <tileset firstgid="2" name="embedded" tilewidth="8" tileheight="8" tilecount="1" columns="1"><image source="local.png" width="8" height="8"/><tile id="0"><objectgroup id="83"><object id="7" x="1" y="0"/></objectgroup></tile></tileset>
 <group id="9" name="Root"><layer id="2" name="Tiles" width="2" height="1"><data encoding="csv">0,1</data></layer><objectgroup id="5" name="Hidden" visible="0"><object id="6" name="old" type="old class" x="3" y="4"><properties><property name="file" type="file" value="script.txt"/></properties></object></objectgroup></group>
 <unknown xmlns:a="urn:test" a:foo="bar"><![CDATA[reserved <data>]]></unknown>
</map>"#).unwrap();
    let mut doc = TmxDocument::open(&source).unwrap();
    assert_eq!(
        doc.tilesets().collect::<Vec<_>>(),
        vec![(1, Some("external.tsx")), (2, None)]
    );
    assert_eq!(doc.tileset_name(2), Some("embedded"));
    assert_eq!(doc.object_layer_id(6), Some(5));
    assert!(doc.object_mut(7).is_none());
    assert!(doc.layer_mut(83).is_none());
    assert!(doc.add_rectangle(2, None).is_err());
    assert!(doc.add_layer(Some(2), LayerKind::Group, "invalid").is_err());
    let group = doc.add_layer(Some(9), LayerKind::Group, "Added").unwrap();
    let layer = doc
        .add_layer(Some(group), LayerKind::Object, "Triggers")
        .unwrap();
    let id = doc.add_rectangle(layer, None).unwrap();
    assert_eq!(id, 7);
    assert!(doc.add_rectangle(layer, Some(6)).is_err());
    let obj = doc.object_mut(id).unwrap();
    obj.set_attribute("name", "a<>&\"\t");
    obj.set_attribute("width", 10.5);
    obj.set_attribute("height", 8);
    obj.set_class("spawn");
    doc.object_mut(6).unwrap().set_class("portal");
    doc.layer_mut(2)
        .unwrap()
        .set_tiles(2, 1, &[0, 0x80000002])
        .unwrap();
    let mut values = Loader::new().load_tmx_map(&source).unwrap().properties;
    values.remove("remove");
    values.insert("direction".into(), PropertyValue::IntValue(2));
    values.insert(
        "multiline".into(),
        PropertyValue::StringValue("hello <world>\nsecond line".into()),
    );
    values.insert(
        "list".into(),
        PropertyValue::ListValue(vec![PropertyValue::ClassValue {
            property_type: "Fx".into(),
            properties: [("file".into(), PropertyValue::FileValue("cloud.xml".into()))]
                .into_iter()
                .collect(),
        }]),
    );
    doc.map_mut().set_properties(&values);
    let target = output.join("new.tmx");
    doc.save(&target).unwrap();
    let first = fs::read_to_string(&target).unwrap();
    doc.save(&target).unwrap();
    assert_eq!(first, fs::read_to_string(&target).unwrap());
    for expected in [
        "<!-- copyright -->",
        "a:foo=\"bar\"",
        "<![CDATA[reserved <data>]]>",
        "propertytype=\"Direction\"",
        "source=\"../local.png\"",
        "../script.txt",
    ] {
        assert!(first.contains(expected), "{expected}");
    }
    let map = Loader::new().load_tmx_map(&target).unwrap();
    let root = map.get_layer(0).unwrap().as_group_layer().unwrap();
    let tiles = root.get_layer(0).unwrap().as_tile_layer().unwrap();
    let cell = tiles.get_tile(1, 0).unwrap();
    assert!(cell.flip_h);
    assert_eq!(cell.get_tileset().name, "embedded");
    let objects = root.get_layer(1).unwrap().as_object_layer().unwrap();
    let portal = objects.get_object(0).unwrap();
    assert_eq!(portal.user_type, "portal");
    assert!(!root.get_layer(1).unwrap().visible);
    assert_ne!(values, map.properties); // Relative nested file reference changed on Save As.
    assert_eq!(map.properties["multiline"], values["multiline"]);
    assert_eq!(
        map.properties["list"],
        PropertyValue::ListValue(vec![PropertyValue::ClassValue {
            property_type: "Fx".into(),
            properties: [(
                "file".into(),
                PropertyValue::FileValue("../cloud.xml".into())
            )]
            .into_iter()
            .collect()
        }])
    );
    let added = root.get_layer(2).unwrap().as_group_layer().unwrap();
    let rect = added
        .get_layer(0)
        .unwrap()
        .as_object_layer()
        .unwrap()
        .get_object(0)
        .unwrap();
    assert_eq!(rect.id(), 7);
    assert_eq!(rect.name, "a<>&\"\t");
    assert_eq!(rect.user_type, "spawn");
}

#[test]
fn rejected_edits_preserve_original_layer_and_destination() {
    let dir = tempfile::tempdir().unwrap();
    let mut doc = TmxDocument::open(Path::new("assets/tiled_base64_zlib_infinite.tmx")).unwrap();
    let layer_id = Loader::new()
        .load_tmx_map("assets/tiled_base64_zlib_infinite.tmx")
        .unwrap()
        .get_layer(0)
        .unwrap()
        .id();
    assert!(
        doc.layer_mut(layer_id)
            .unwrap()
            .set_tiles(1, 1, &[0])
            .is_err()
    );
    let target = dir.path().join("level.tmx");
    doc.save(&target).unwrap();
    assert!(Loader::new().load_tmx_map(&target).unwrap().infinite());
    let original = fs::read(&target).unwrap();
    doc.map_mut().set_attribute("name", "invalid\0name");
    assert!(doc.save(&target).is_err());
    assert_eq!(fs::read(&target).unwrap(), original);
}

#[test]
fn delete_only_the_matching_map_object() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("level.tmx");
    fs::write(&path,r#"<map version="1.10" orientation="orthogonal" width="1" height="1" tilewidth="1" tileheight="1" nextobjectid="21">
        <tileset firstgid="1" name="tiles" tilewidth="1" tileheight="1"><tile id="0"><objectgroup><object id="13" x="0" y="0"/></objectgroup></tile></tileset>
        <group><objectgroup><object id="13" name="remove" x="1" y="1"/><object id="20" name="keep" x="0" y="0"/></objectgroup></group></map>"#).unwrap();
    let mut doc = TmxDocument::open(&path).unwrap();
    assert!(doc.remove_object(13));
    assert!(!doc.remove_object(13));
    doc.save(&path).unwrap();
    let xml = fs::read_to_string(&path).unwrap();
    assert!(!xml.contains("name=\"remove\""));
    assert!(xml.contains("name=\"keep\""));
    assert_eq!(xml.matches("id=\"13\"").count(), 1);
    assert!(xml.contains("nextobjectid=\"21\""));
}
