# Как написать свой WASM-парсер для chapdownloader

Эта инструкция поможет вам создать собственный парсер глав для любого сайта, используя Rust и WebAssembly. Такой парсер можно подключить к chapdownloader через интерфейс "Другой сайт".

---

## 1. Установите необходимые инструменты

- Rust: https://www.rust-lang.org/tools/install
- wasm32-unknown-unknown target:
  ```sh
  rustup target add wasm32-unknown-unknown
  ```
- (Рекомендуется) VSCode или любой другой редактор

---

## 2. Создайте новый проект

```sh
cargo new my_parser
cd my_parser
```

---

## 3. Настройте Cargo.toml

В секции `[lib]` добавьте:
```toml
[lib]
crate-type = ["cdylib"]
```

В секции `[dependencies]` добавьте:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 4. Пример кода парсера (src/lib.rs)

```rust
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
    let html_bytes = unsafe { std::slice::from_raw_parts(html_ptr, html_len) };
    let html = match std::str::from_utf8(html_bytes) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let mut images = Vec::new();
    // TODO: Здесь ваша логика парсинга HTML!
    // Пример: ищем все <img ... data-lazy-src="..."> внутри <div id="readerarea">...</div>
    // ...
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
```

---

## 5. Реализуйте свою логику парсинга

- В функции `parse_chapter_images` напишите код, который из HTML главы достаёт ссылки на все изображения страниц.
- Верните результат в виде JSON-массива строк (пример: `["https://.../01.webp", "https://.../02.webp"]`).

---

## 6. Сборка парсера

```sh
cargo build --target wasm32-unknown-unknown --release
```

Готовый файл будет лежать по пути:
```
target/wasm32-unknown-unknown/release/my_parser.wasm
```

---

## 7. Использование в chapdownloader

1. Зайдите на сайт/интерфейс chapdownloader.
2. Выберите "Другой сайт".
3. Загрузите ваш `.wasm` файл.
4. Введите ссылку на главу и нажмите "Загрузить парсер".

---

## 8. Советы и рекомендации

- Для парсинга HTML используйте простые методы поиска по строке (`find`, `split` и т.д.), либо напишите свой мини-парсер.
- Не используйте внешние библиотеки, которые не поддерживаются в WASM без std (например, `scraper`, `regex` — только если вы умеете собирать их для wasm32-unknown-unknown).
- Всегда возвращайте результат в виде JSON-массива строк.
- Не забывайте про функции `alloc` и `dealloc` — они обязательны для работы с памятью.

---

## 9. Пример для сайта Nightscans

```rust
// ...см. nightscans_parser/src/lib.rs в этом репозитории...
```

---

## 10. Вопросы и помощь

Если возникли вопросы — создайте issue на GitHub или напишите в Telegram. 