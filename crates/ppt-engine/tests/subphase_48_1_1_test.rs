#![forbid(unsafe_code)]
//! Sub-Phase 48.1.1 Integration Tests: Interactive Slide Master & Layout Designer View
//! GEC-v7.0 Strict Clean-Room Test Suite

use ppt_engine::master::SlideMasterDesigner;
use ppt_engine::package::PresentationPackage;

#[test]
fn test_1_extract_master_tree_from_blank_package() {
    let pkg = PresentationPackage::create_empty();
    let tree = SlideMasterDesigner::extract_master_tree(&pkg);

    assert_eq!(tree.masters.len(), 1, "Should have 1 slide master");
    let master = &tree.masters[0];
    assert_eq!(master.part_name, "ppt/slideMasters/slideMaster1.xml");
    assert!(
        !master.layouts.is_empty(),
        "Master should link to at least 1 layout"
    );
    assert_eq!(tree.slide_size.cx, 12_192_000);
    assert_eq!(tree.slide_size.cy, 6_858_000);
}

#[test]
fn test_2_create_custom_slide_layout() {
    let mut pkg = PresentationPackage::create_empty();
    let master_part = "ppt/slideMasters/slideMaster1.xml";

    let new_layout_part = SlideMasterDesigner::create_layout(
        &mut pkg,
        master_part,
        "twoContent",
        "Custom Two Column Layout",
    )
    .expect("Should create custom layout successfully");

    assert!(new_layout_part.starts_with("ppt/slideLayouts/slideLayout"));
    assert!(pkg.slide_layouts.contains_key(&new_layout_part));

    let layout = pkg
        .slide_layouts
        .get(&new_layout_part)
        .expect("Layout must exist in package");
    assert_eq!(
        layout.matching_name.as_deref(),
        Some("Custom Two Column Layout")
    );
    assert_eq!(layout.layout_type.as_deref(), Some("twoContent"));

    // Verify shapes were generated for twoContent layout (Title, Left Content, Right Content + 3 Footers)
    let placeholders =
        SlideMasterDesigner::extract_placeholders(&layout.common_slide_data.shape_tree.shapes);
    assert!(placeholders.iter().any(|p| p.ph_type == "title"));
    assert!(placeholders
        .iter()
        .any(|p| p.ph_type == "body" && p.index == Some(1)));
    assert!(placeholders
        .iter()
        .any(|p| p.ph_type == "body" && p.index == Some(2)));
}

#[test]
fn test_3_standard_layout_presets_coverage() {
    let presets = SlideMasterDesigner::standard_layout_presets();
    assert_eq!(presets.len(), 9);

    let preset_ids: Vec<&str> = presets.iter().map(|(id, _)| *id).collect();
    assert!(preset_ids.contains(&"title"));
    assert!(preset_ids.contains(&"titleAndContent"));
    assert!(preset_ids.contains(&"sectionHeader"));
    assert!(preset_ids.contains(&"twoContent"));
    assert!(preset_ids.contains(&"comparison"));
    assert!(preset_ids.contains(&"titleOnly"));
    assert!(preset_ids.contains(&"blank"));
    assert!(preset_ids.contains(&"contentWithCaption"));
    assert!(preset_ids.contains(&"pictureWithCaption"));
}

#[test]
fn test_4_duplicate_slide_layout() {
    let mut pkg = PresentationPackage::create_empty();
    let original_layout_part = "ppt/slideLayouts/slideLayout1.xml";

    let dup_part = SlideMasterDesigner::duplicate_layout(
        &mut pkg,
        original_layout_part,
        "Duplicated Title Slide",
    )
    .expect("Should duplicate layout");

    assert_ne!(dup_part, original_layout_part);
    assert!(pkg.slide_layouts.contains_key(&dup_part));

    let dup_layout = pkg
        .slide_layouts
        .get(&dup_part)
        .expect("Duplicated layout part exists");
    assert_eq!(
        dup_layout.matching_name.as_deref(),
        Some("Duplicated Title Slide")
    );
}

#[test]
fn test_5_delete_slide_layout_safe() {
    let mut pkg = PresentationPackage::create_empty();
    let master_part = "ppt/slideMasters/slideMaster1.xml";

    let new_layout_part = SlideMasterDesigner::create_layout(
        &mut pkg,
        master_part,
        "blank",
        "Temporary Blank Layout",
    )
    .expect("Created layout");

    assert!(pkg.slide_layouts.contains_key(&new_layout_part));

    SlideMasterDesigner::delete_layout(&mut pkg, &new_layout_part)
        .expect("Should safely delete layout");

    assert!(!pkg.slide_layouts.contains_key(&new_layout_part));
}

#[test]
fn test_6_add_and_remove_placeholder_on_layout() {
    let mut pkg = PresentationPackage::create_empty();
    let layout_part = "ppt/slideLayouts/slideLayout1.xml";

    let ph_id = SlideMasterDesigner::add_placeholder(
        &mut pkg,
        layout_part,
        "pic",
        1_000_000,
        2_000_000,
        4_000_000,
        3_000_000,
    )
    .expect("Should add picture placeholder");

    let layout = pkg.slide_layouts.get(layout_part).expect("Layout exists");
    let placeholders =
        SlideMasterDesigner::extract_placeholders(&layout.common_slide_data.shape_tree.shapes);
    let added = placeholders.iter().find(|p| p.shape_id == ph_id);
    assert!(added.is_some(), "Added placeholder should be present");
    let added_ph = added.expect("Valid placeholder");
    assert_eq!(added_ph.ph_type, "pic");
    assert_eq!(added_ph.x, 1_000_000);
    assert_eq!(added_ph.y, 2_000_000);

    // Remove the placeholder
    SlideMasterDesigner::remove_placeholder(&mut pkg, layout_part, ph_id)
        .expect("Should remove placeholder");

    let layout_after = pkg.slide_layouts.get(layout_part).expect("Layout exists");
    let phs_after = SlideMasterDesigner::extract_placeholders(
        &layout_after.common_slide_data.shape_tree.shapes,
    );
    assert!(!phs_after.iter().any(|p| p.shape_id == ph_id));
}

#[test]
fn test_7_update_placeholder_transform() {
    let mut pkg = PresentationPackage::create_empty();
    let layout_part = "ppt/slideLayouts/slideLayout1.xml";

    // Extract first placeholder id
    let layout = pkg.slide_layouts.get(layout_part).expect("Layout exists");
    let placeholders =
        SlideMasterDesigner::extract_placeholders(&layout.common_slide_data.shape_tree.shapes);
    assert!(!placeholders.is_empty(), "Must have placeholders");
    let target_id = placeholders[0].shape_id;

    SlideMasterDesigner::update_placeholder_transform(
        &mut pkg,
        layout_part,
        target_id,
        500_000,
        600_000,
        8_000_000,
        2_000_000,
    )
    .expect("Should update placeholder transform");

    let layout_after = pkg.slide_layouts.get(layout_part).expect("Layout exists");
    let updated_phs = SlideMasterDesigner::extract_placeholders(
        &layout_after.common_slide_data.shape_tree.shapes,
    );
    let target_after = updated_phs
        .iter()
        .find(|p| p.shape_id == target_id)
        .expect("Placeholder exists");
    assert_eq!(target_after.x, 500_000);
    assert_eq!(target_after.y, 600_000);
    assert_eq!(target_after.cx, 8_000_000);
    assert_eq!(target_after.cy, 2_000_000);
}

#[test]
fn test_8_apply_layout_to_slide() {
    let mut pkg = PresentationPackage::create_empty();
    let master_part = "ppt/slideMasters/slideMaster1.xml";

    let custom_layout_part = SlideMasterDesigner::create_layout(
        &mut pkg,
        master_part,
        "twoContent",
        "Two Content for Slide 0",
    )
    .expect("Layout created");

    SlideMasterDesigner::apply_layout_to_slide(&mut pkg, 0, &custom_layout_part)
        .expect("Should apply layout to slide 0");

    let resolved_part = SlideMasterDesigner::get_slide_layout_part(&pkg, 0)
        .expect("Slide should have resolved layout part");
    assert_eq!(resolved_part, custom_layout_part);
}

#[test]
fn test_9_get_slide_layout_part_resolution() {
    let pkg = PresentationPackage::create_empty();
    let layout_part = SlideMasterDesigner::get_slide_layout_part(&pkg, 0)
        .expect("Slide 0 layout part should resolve");
    assert!(layout_part.contains("slideLayout1.xml"));
}

#[test]
fn test_10_roundtrip_save_and_reopen_with_designer_modifications() {
    let mut pkg = PresentationPackage::create_empty();
    let master_part = "ppt/slideMasters/slideMaster1.xml";

    // 1. Create two custom layouts
    let l1 = SlideMasterDesigner::create_layout(
        &mut pkg,
        master_part,
        "sectionHeader",
        "Section Divider",
    )
    .expect("Created l1");
    let l2 = SlideMasterDesigner::create_layout(
        &mut pkg,
        master_part,
        "comparison",
        "Product Comparison",
    )
    .expect("Created l2");

    // 2. Add custom placeholder on l2
    let pic_id = SlideMasterDesigner::add_placeholder(
        &mut pkg, &l2, "pic", 500_000, 500_000, 3_000_000, 3_000_000,
    )
    .expect("Added placeholder");

    // 3. Apply l1 to slide 0
    SlideMasterDesigner::apply_layout_to_slide(&mut pkg, 0, &l1).expect("Applied l1 to slide 0");

    // 4. Serialize to OPC package zip bytes
    let mut buf = std::io::Cursor::new(Vec::new());
    pkg.save(&mut buf).expect("Save package");
    let raw_bytes = buf.into_inner();
    assert!(raw_bytes.starts_with(b"PK"), "Must be valid ZIP package");

    // 5. Reopen package from bytes
    let reopened =
        PresentationPackage::open(std::io::Cursor::new(&raw_bytes)).expect("Reopen package");

    // 6. Verify master tree
    let tree = SlideMasterDesigner::extract_master_tree(&reopened);
    assert_eq!(tree.masters.len(), 1);
    let master = &tree.masters[0];

    // Verify custom layouts exist in reopened package
    let layout_parts: Vec<&str> = master
        .layouts
        .iter()
        .map(|l| l.part_name.as_str())
        .collect();
    assert!(
        layout_parts.contains(&l1.as_str()),
        "Reopened package must contain layout l1"
    );
    assert!(
        layout_parts.contains(&l2.as_str()),
        "Reopened package must contain layout l2"
    );

    // Verify slide 0 layout relationship points to l1
    let slide0_layout = SlideMasterDesigner::get_slide_layout_part(&reopened, 0)
        .expect("Resolved slide 0 layout in reopened package");
    assert_eq!(slide0_layout, l1);

    // Verify added picture placeholder on l2 persisted
    let reopened_l2 = reopened.slide_layouts.get(&l2).expect("Reopened l2 exists");
    let reopened_phs =
        SlideMasterDesigner::extract_placeholders(&reopened_l2.common_slide_data.shape_tree.shapes);
    assert!(reopened_phs
        .iter()
        .any(|p| p.shape_id == pic_id && p.ph_type == "pic"));
}
