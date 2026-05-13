# FFI Image Processor

CLI-приложение на Rust, которое загружает PNG-изображение, динамически подключает плагин через FFI и сохраняет обработанный результат.

## Структура

- `image_processor` — основное CLI-приложение.
- `mirror_plugin` — плагин зеркального отражения.
- `blur_plugin` — плагин размытия.

## Сборка

```bash
cargo build --workspace
```

После сборки динамические библиотеки будут доступны в `target/debug`:

- Linux: `libmirror.so`, `libblur.so`
- macOS: `libmirror.dylib`, `libblur.dylib`
- Windows: `mirror.dll`, `blur.dll`

## Использование

```bash
cargo run -p image_processor -- \
  input.png \
  output.png \
  mirror \
  mirror_params.json \
  --plugin-path target/debug
```

Аргументы:

- `input` — путь к исходному PNG.
- `output` — путь к выходному PNG.
- `plugin` — имя библиотеки без расширения, например `mirror` или `blur`.
- `params` — путь к текстовому файлу с параметрами.
- `--plugin-path` — директория с собранными плагинами, по умолчанию `target/debug`.

## Формат параметров

Плагины читают параметры из JSON-файла.

### Mirror

`mirror_params.json`

```json
{
  "horizontal": true,
  "vertical": false
}
```

### Blur

`blur_params.json`

```json
{
  "radius": 2,
  "iterations": 2
}
```

## Проверка

```bash
cargo test --workspace
```

Примеры негативных сценариев для ручной проверки:

- несуществующий `input`;
- несуществующий `params`;
- неверное имя плагина;
- повреждённый PNG-файл.
