use druid::widget::{Button, CrossAxisAlignment, Flex, Label, List, Scroll, TextBox, Container};
use druid::{AppLauncher, Data, Lens, Widget, WidgetExt, WindowDesc, Color};
use std::fs;
use std::process::Command;
use std::sync::Arc;
use rfd::FileDialog;
use std::path::{PathBuf, Path};
use walkdir::WalkDir;

#[derive(Clone, Data, Lens)]
struct AppState {
    selected_folder: String,
    files: Arc<Vec<FileItem>>,
    search_enabled: bool,
    search_query: String,
}

#[derive(Clone, Data)]
struct FileItem {
    name: String,
    path: String,
    relative_path: String, // Добавлено поле для относительного пути
}

fn build_ui() -> impl Widget<AppState> {
    let select_folder_button = Button::new("📁")
        .on_click(|_, data: &mut AppState, _| {
            if let Some(folder) = FileDialog::new().pick_folder() {
                data.selected_folder = folder.to_string_lossy().to_string();
                data.search_enabled = true;
            }
        })
        .padding(5.0)
        .fix_width(100.0)
        .background(Color::rgb8(100, 149, 237)) // Цвет кнопки
        .rounded(5.0) // Закругленные углы
        .center();

    let search_box = TextBox::new()
        .with_placeholder("Введите имя файла")
        .lens(AppState::search_query)
        .padding(10.0)
        .fix_width(200.0)
        .border(Color::rgb8(150, 150, 150), 1.0) // Изменено на 1.0 для совпадения с толщиной рамки кнопки удаления
        .center();

    let search_button = Button::new("🔍")
        .on_click(|ctx, data: &mut AppState, _| {
            data.files = search_files(&data.selected_folder, &data.search_query);
            ctx.request_update();
        })
        .disabled_if(|data: &AppState, _| !data.search_enabled)
        .padding(5.0)
        .fix_width(100.0)
        .background(Color::rgb8(100, 237, 149)) // Цвет кнопки
        .rounded(5.0) // Закругленные углы
        .center();

    let folder_display = Container::new(Label::dynamic(|data: &AppState, _| {
        if data.selected_folder.is_empty() {
            "Директория не выбрана".to_string()
        } else {
            format!("Выбрана директория: {}", data.selected_folder)
        }
    }))
    .border(Color::rgb8(150, 150, 150), 1.0) // Изменено на 1.0 для совпадения с толщиной рамки кнопки удаления
    .padding(10.0)
    .fix_width(400.0)
    .center();

    let file_list = Scroll::new(List::new(|| {
        Flex::row()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(Label::dynamic(|item: &FileItem, _| item.relative_path.clone())
                .padding(5.0)
                .center()
                .background(Color::rgba8(240, 240, 240, 128)) // Цвет фона элемента списка
                .rounded(5.0)) // Закругленные углы
            .with_spacer(10.0)
            .with_child(
                Button::new("📂")  // Открыть путь к файлу
                    .on_click(|_, item: &mut FileItem, _| {
                        open_folder(&item.path);
                    })
                    .padding(5.0)
                    .fix_width(100.0)
                    .background(Color::rgb8(100, 237, 149)) // Цвет кнопки
                    .rounded(5.0)
                    .center()
            )
            .with_spacer(5.0)
            .with_child(
                Button::new("🗑")  // Удалить файл
                    .on_click(|_, item: &mut FileItem, _| {
                        delete_file(&item.path);
                    })
                    .padding(5.0)
                    .fix_width(100.0)
                    .background(Color::rgb8(237, 100, 100)) // Цвет кнопки
                    .rounded(5.0)
                    .center()
            )
    }))
    .lens(AppState::files)
    .padding(5.0);

    // Создаем контейнер с фоновым цветом
    let background = Container::new(
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(
                Flex::row()
                    .with_child(select_folder_button)
                    .with_spacer(10.0)
                    .with_child(search_box)
                    .with_spacer(10.0)
                    .with_child(search_button)
                    .padding(5.0)
            )
            .with_spacer(10.0)
            .with_child(folder_display) // Отображение директории
            .with_flex_child(file_list, 1.0)
            .padding(5.0)
    )
    .background(Color::rgb8(126, 91, 155)) // Задаем цвет фона
    .padding(5.0); // Отступ вокруг контейнера
    background
}
fn search_files(path: &str, query: &str) -> Arc<Vec<FileItem>> {
    let root_path = Path::new(path).to_path_buf(); // Сохраняем путь к корневой директории
    let files = WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok()) // Фильтрация ошибок
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let name = path.file_name()?.to_string_lossy().to_string();

            // Определяем относительный путь в зависимости от уровня вложенности
            let relative_path = if path.parent() == Some(&root_path) {
                name.clone() // Если в корневой директории, сохраняем только имя
            } else {
                path.strip_prefix(&root_path).ok()?.to_string_lossy().to_string() // Иначе - относительный путь
            };

            Some(FileItem {
                name: name.clone(),
                path: path.to_string_lossy().to_string(),
                relative_path, // Запоминаем относительный путь
            })
        })
        .filter(|file| query.is_empty() || file.name.contains(query))
        .collect::<Vec<_>>();

    Arc::new(files)
}

fn open_folder(file_path: &str) {
    let path = PathBuf::from(file_path);
    if let Some(parent) = path.parent() {
        if cfg!(target_os = "windows") {
            let _ = Command::new("explorer").arg(parent).spawn();
        } else if cfg!(target_os = "macos") {
            let _ = Command::new("open").arg(parent).spawn();
        } else {
            let _ = Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

fn delete_file(file_path: &str) {
    if let Err(e) = fs::remove_file(file_path) {
        println!("Ошибка при удалении файла {}: {}", file_path, e);
    } else {
        println!("Файл {} удален", file_path);
    }
}

#[cfg(target_os = "windows")]
fn hide_console_window() {
    #[cfg(not(debug_assertions))]
    {
        use winapi::um::wincon::FreeConsole;
        unsafe {
            FreeConsole();
        }
    }
}

fn main() {
    // Скрываем консоль только для релизной версии на Windows
    #[cfg(target_os = "windows")]
    hide_console_window();

    let main_window = WindowDesc::new(build_ui())
        .title("File Finder")
        .window_size((1100.0, 900.0));

    let initial_state = AppState {
        selected_folder: "".to_string(),
        files: Arc::new(Vec::new()),
        search_enabled: false,
        search_query: "".to_string(),
    };

    AppLauncher::with_window(main_window)
        // Включаем логирование только для отладочной сборки
        .log_to_console_if_debug()
        .launch(initial_state)
        .expect("Не удалось запустить приложение");
}

trait AppLauncherExt {
    fn log_to_console_if_debug(self) -> Self;
}

impl AppLauncherExt for AppLauncher<AppState> {
    fn log_to_console_if_debug(self) -> Self {
        #[cfg(debug_assertions)]
        {
            self.log_to_console()
        }

        #[cfg(not(debug_assertions))]
        {
            self
        }
    }
}
