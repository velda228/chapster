use std::alloc::{alloc as std_alloc, dealloc as std_dealloc, Layout};
use std::ptr;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut u8 {
    let layout = Layout::from_size_align(size, 8).unwrap();
    unsafe { std_alloc(layout) }
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
    let layout = Layout::from_size_align(size, 8).unwrap();
    unsafe { std_dealloc(ptr, layout) }
}

#[no_mangle]
pub extern "C" fn parse_chapter_images(html_ptr: *const u8, html_len: usize) -> *mut u8 {
    // Читаем HTML из памяти
    let html_bytes = unsafe { std::slice::from_raw_parts(html_ptr, html_len) };
    let html = match std::str::from_utf8(html_bytes) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut images = Vec::new();

    // 1. Найти <div id="readerarea">
    if let Some(reader_start) = html.find("<div id=\"readerarea\"") {
        // Найти начало содержимого (после >)
        let after_div = match html[reader_start..].find('>') {
            Some(pos) => reader_start + pos + 1,
            None => return ptr::null_mut(),
        };
        // Найти конец блока (</div>)
        let after_end = match html[after_div..].find("</div>") {
            Some(pos) => after_div + pos,
            None => html.len(),
        };
        let reader_content = &html[after_div..after_end];

        // 2. Внутри reader_content искать только <img ... data-lazy-src="...">
        let mut pos = 0;
        while pos < reader_content.len() {
            if let Some(img_start) = reader_content[pos..].find("<img") {
                let img_start = pos + img_start;
                if let Some(img_end) = reader_content[img_start..].find('>') {
                    let img_tag = &reader_content[img_start..img_start + img_end + 1];
                    // Только data-lazy-src
                    if let Some(src_start) = img_tag.find("data-lazy-src=\"") {
                        let src_start = src_start + 15;
                        if let Some(src_end) = img_tag[src_start..].find('"') {
                            let url = &img_tag[src_start..src_start + src_end];
                            if !url.is_empty() && !url.starts_with("data:") {
                                images.push(url.to_string());
                            }
                        }
                    }
                    pos = img_start + img_end + 1;
                } else {
                    pos += 1;
                }
            } else {
                break;
            }
        }
    }
    // 3. Если ничего не нашли — пустой массив
    images.sort();
    images.dedup();
    let json_result = match serde_json::to_string(&images) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let result_bytes = json_result.as_bytes();
    let result_ptr = alloc(result_bytes.len() + 1);
    unsafe {
        ptr::copy_nonoverlapping(result_bytes.as_ptr(), result_ptr, result_bytes.len());
        ptr::write(result_ptr.add(result_bytes.len()), 0);
    }
    result_ptr
} 