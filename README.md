# Notes - Приложение для заметок

Простое и удобное приложение для ведения заметок с поддержкой групп и тем оформления.

![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg)
![GUI: egui](https://img.shields.io/badge/GUI-egui-blue.svg)
![Platform: Cross-platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-green.svg)

## Особенности

- 📝 **Создание и редактирование заметок** с простым интерфейсом
- 📁 **Организация по группам** с поддержкой вложенности
- 📌 **Закрепление важных заметок**
- 🎨 **Темы оформления** (светлая, тёмная, автоматическая)
- 📋 **Копирование в буфер обмена**
- 💾 **Автоматическое сохранение**
- 🗃️ **Постоянное поле для черновиков**
- 🔄 **Динамическое изменение размера панелей**

## Скриншоты

*Интерфейс приложения с группами заметок и тёмной темой*

## Установка

### Скачать готовую сборку

1. Перейдите в Releases на одной из платформ:
   - **GitHub**: [Releases](https://github.com/your-username/notes/releases)
   - **GitFlic**: [Releases](https://gitflic.ru/your-username/notes/releases)
2. Скачайте версию для вашей ОС:
   - `Notes-windows-x64.exe` - для Windows
   - `Notes-linux-x64` - для Linux  
   - `Notes-macos-intel` - для macOS Intel
   - `Notes-macos-apple-silicon` - для macOS Apple Silicon

### Сборка из исходников

```bash
# Клонирование репозитория (GitHub)
git clone https://github.com/your-username/notes.git
cd notes

# Или с GitFlic
git clone https://gitflic.ru/your-username/notes.git
cd notes

# Сборка
cargo build --release

# Запуск
cargo run --release
```

Подробная инструкция по сборке: [BUILD.md](BUILD.md)

## Использование

### Основные функции

1. **Создание заметки**: Нажмите "Новая заметка", введите заголовок и содержимое
2. **Редактирование**: Выберите заметку и нажмите кнопку "Редактировать"
3. **Группировка**: Создайте группу и перетащите заметки или назначьте группу при создании
4. **Закрепление**: Нажмите 📌 рядом с заголовком заметки
5. **Копирование**: Используйте кнопки "Копировать" или "В заметки"

### Горячие клавиши

- `Ctrl+N` - Новая заметка
- `Ctrl+S` - Сохранить изменения
- `Ctrl+C` - Копировать выделенный текст
- `Delete` - Удалить выбранную заметку

### Файлы данных

Заметки сохраняются в:
- **Windows**: `%USERPROFILE%\.notes\notes\`
- **Linux**: `~/.notes/notes/`
- **macOS**: `~/.notes/notes/`

## Технические детали

- **Язык**: Rust 🦀
- **GUI Framework**: egui/eframe
- **Сохранение**: JSON файлы
- **Поддерживаемые ОС**: Windows 10+, Linux, macOS 10.15+

## Разработка

### Требования

- Rust 1.75+
- Cargo

### Структура проекта

```
src/
├── main.rs          # Точка входа
├── app.rs           # Основная логика приложения
├── notes.rs         # Работа с заметками
├── error.rs         # Обработка ошибок
└── assets/          # Ресурсы приложения
```

### Сборка для разработки

```bash
# Режим отладки
cargo run

# С логами
RUST_LOG=debug cargo run

# Тесты
cargo test
```



## Планы развития

- [ ] Синхронизация между устройствами
- [ ] Поиск по заметкам
- [ ] Экспорт в Markdown/PDF
- [ ] Плагины и расширения
- [ ] Мобильная версия
- [ ] SSH синхронизация

## Вклад в проект

Приветствуются любые улучшения! 

1. Fork репозитория
2. Создайте ветку для новой функции (`git checkout -b feature/amazing-feature`)
3. Зафиксируйте изменения (`git commit -m 'Add amazing feature'`)
4. Push в ветку (`git push origin feature/amazing-feature`)
5. Создайте Pull Request

## Лицензия

Этот проект распространяется под лицензией MIT. См. файл [LICENSE](LICENSE) для подробностей.

## Авторы

- **Ваше имя** - *Основной разработчик* - [GitHub](https://github.com/yourusername)

## Благодарности

- [egui](https://github.com/emilk/egui) - За отличный GUI фреймворк
- [Rust сообщество](https://www.rust-lang.org/community) - За поддержку и вдохновение
- Всем тестировщикам и пользователям!

---

⭐ **Если проект вам понравился, поставьте звёздочку!** ⭐ 