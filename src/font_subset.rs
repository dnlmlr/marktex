use allsorts::{
    binary::read::ReadScope,
    font::read_cmap_subtable,
    subset::subset,
    tables::{cmap::Cmap, FontTableProvider},
    tag,
};
use anyhow::Result;

pub fn font_subset(font_data: &[u8], reference_text: &str) -> Result<Vec<u8>> {
    let font_file = ReadScope::new(font_data).read::<allsorts::font_data::FontData<'_>>()?;
    let provider = font_file.table_provider(0)?;

    let cmap_data = provider.read_table_data(tag::CMAP).unwrap();
    let cmap = ReadScope::new(&cmap_data).read::<Cmap<'_>>()?;
    let (_, cmap_subtable) = read_cmap_subtable(&cmap).unwrap().unwrap();

    let mut glyph_ids = vec![
        // GlphyID 0 is the unknown character and must always be set
        0,
        // Prevent `allsorts` from using MacRoman encoding by using a non supported character
        cmap_subtable
            .map_glyph('€' as u32)?
            .ok_or(anyhow::anyhow!("'€' not included in font"))?,
        // GenPDF lists use this so called "em dash" which is not actually a normal "hyphen" (-)
        cmap_subtable
            .map_glyph('–' as u32)?
            .ok_or(anyhow::anyhow!("'–' (em dash) not included in font"))?,
    ];

    glyph_ids.extend(
        reference_text
            .chars()
            .flat_map(|ch| cmap_subtable.map_glyph(ch as u32).ok().flatten()),
    );
    glyph_ids.sort();
    glyph_ids.dedup();

    let new_font = subset(&provider, &glyph_ids)?;

    Ok(new_font)
}
