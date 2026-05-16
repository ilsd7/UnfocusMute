#![cfg_attr(not(windows), allow(dead_code))]

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    #[default]
    Ko,
    En,
    Ja,
    ZhHans,
}

impl Language {
    pub const ALL: [Language; 4] = [Language::Ko, Language::En, Language::Ja, Language::ZhHans];

    pub fn native_name(self) -> &'static str {
        match self {
            Language::Ko => "한국어",
            Language::En => "English",
            Language::Ja => "日本語",
            Language::ZhHans => "简体中文",
        }
    }

    pub fn strings(self) -> Strings {
        match self {
            Language::Ko => Strings {
                app_title: "UnfocusMute",
                status_running: "감시 중",
                status_paused: "일시 중지됨",
                onboarding_title: "처음 설정",
                onboarding_body: "1. 음소거할 게임이나 앱을 먼저 실행하세요.\r\n2. 실행 중인 프로세스에서 선택해 등록합니다.\r\n3. 창을 닫아도 트레이에서 계속 감시합니다.",
                onboarding_done: "안내 완료",
                registered_processes: "등록된 앱",
                running_processes: "실행 중인 앱",
                add_process_section: "앱 등록",
                running_process_hint: "목록을 새로 고친 뒤 음소거할 앱을 선택하세요.",
                manual_process: "직접 입력",
                add_selected: "선택 추가",
                add_manual: "직접 추가",
                remove_selected: "선택 제거",
                pause: "일시 중지",
                resume: "다시 시작",
                start_minimized: "시작 시 트레이로 최소화",
                launch_on_startup: "Windows 로그인 시 실행",
                restore_on_exit: "종료 시 앱 음소거 해제",
                language: "언어",
                refresh: "새로 고침",
                quit: "종료",
                show: "열기",
                hide: "트레이로 숨기기",
                manual_placeholder: "예: game.exe",
                settings_title: "동작 설정",
                no_targets: "아직 등록된 앱이 없습니다.",
                target_count: "등록",
                muted_count: "앱 음소거",
                polling_interval: "확인 간격",
                unsupported_os: "UnfocusMute는 Windows 11용 앱입니다. 이 환경에서는 코어 테스트와 문서 작업만 실행할 수 있습니다.",
            },
            Language::En => Strings {
                app_title: "UnfocusMute",
                status_running: "Watching",
                status_paused: "Paused",
                onboarding_title: "First setup",
                onboarding_body: "1. Start the game or app you want to mute.\r\n2. Pick it from running processes and add it.\r\n3. Closing the window keeps monitoring in the tray.",
                onboarding_done: "Done",
                registered_processes: "Registered apps",
                running_processes: "Running apps",
                add_process_section: "Add an app",
                running_process_hint: "Refresh the list, then choose the app to mute.",
                manual_process: "Manual entry",
                add_selected: "Add selected",
                add_manual: "Add manually",
                remove_selected: "Remove selected",
                pause: "Pause",
                resume: "Resume",
                start_minimized: "Start minimized to tray",
                launch_on_startup: "Run at Windows sign-in",
                restore_on_exit: "Unmute apps on exit",
                language: "Language",
                refresh: "Refresh",
                quit: "Quit",
                show: "Open",
                hide: "Hide to tray",
                manual_placeholder: "Example: game.exe",
                settings_title: "Behavior",
                no_targets: "No apps are registered yet.",
                target_count: "Registered",
                muted_count: "Muted by app",
                polling_interval: "Poll interval",
                unsupported_os: "UnfocusMute is a Windows 11 app. This environment can run only core tests and documentation tasks.",
            },
            Language::Ja => Strings {
                app_title: "UnfocusMute",
                status_running: "監視中",
                status_paused: "一時停止",
                onboarding_title: "初期設定",
                onboarding_body: "1. ミュートしたいゲームやアプリを先に起動します。\r\n2. 実行中のプロセスから選択して登録します。\r\n3. ウィンドウを閉じてもトレイで監視を続けます。",
                onboarding_done: "完了",
                registered_processes: "登録済みアプリ",
                running_processes: "実行中のアプリ",
                add_process_section: "アプリ登録",
                running_process_hint: "一覧を更新して、ミュートしたいアプリを選択します。",
                manual_process: "直接入力",
                add_selected: "選択を追加",
                add_manual: "手動で追加",
                remove_selected: "選択を削除",
                pause: "一時停止",
                resume: "再開",
                start_minimized: "起動時にトレイへ最小化",
                launch_on_startup: "Windowsサインイン時に実行",
                restore_on_exit: "終了時にミュート解除",
                language: "言語",
                refresh: "更新",
                quit: "終了",
                show: "開く",
                hide: "トレイへ隠す",
                manual_placeholder: "例: game.exe",
                settings_title: "動作設定",
                no_targets: "登録済みアプリはまだありません。",
                target_count: "登録",
                muted_count: "アプリでミュート",
                polling_interval: "確認間隔",
                unsupported_os: "UnfocusMuteはWindows 11向けアプリです。この環境ではコアテストとドキュメント作業のみ実行できます。",
            },
            Language::ZhHans => Strings {
                app_title: "UnfocusMute",
                status_running: "监视中",
                status_paused: "已暂停",
                onboarding_title: "首次设置",
                onboarding_body: "1. 先启动要静音的游戏或应用。\r\n2. 从正在运行的进程中选择并添加。\r\n3. 关闭窗口后仍会在托盘继续监视。",
                onboarding_done: "完成",
                registered_processes: "已注册应用",
                running_processes: "正在运行的应用",
                add_process_section: "添加应用",
                running_process_hint: "刷新列表，然后选择要静音的应用。",
                manual_process: "手动输入",
                add_selected: "添加所选",
                add_manual: "手动添加",
                remove_selected: "移除所选",
                pause: "暂停",
                resume: "继续",
                start_minimized: "启动时最小化到托盘",
                launch_on_startup: "Windows 登录时运行",
                restore_on_exit: "退出时取消静音",
                language: "语言",
                refresh: "刷新",
                quit: "退出",
                show: "打开",
                hide: "隐藏到托盘",
                manual_placeholder: "例如: game.exe",
                settings_title: "行为设置",
                no_targets: "尚未注册任何应用。",
                target_count: "已注册",
                muted_count: "应用已静音",
                polling_interval: "检查间隔",
                unsupported_os: "UnfocusMute 是 Windows 11 应用。此环境只能运行核心测试和文档任务。",
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Strings {
    pub app_title: &'static str,
    pub status_running: &'static str,
    pub status_paused: &'static str,
    pub onboarding_title: &'static str,
    pub onboarding_body: &'static str,
    pub onboarding_done: &'static str,
    pub registered_processes: &'static str,
    pub running_processes: &'static str,
    pub add_process_section: &'static str,
    pub running_process_hint: &'static str,
    pub manual_process: &'static str,
    pub add_selected: &'static str,
    pub add_manual: &'static str,
    pub remove_selected: &'static str,
    pub pause: &'static str,
    pub resume: &'static str,
    pub start_minimized: &'static str,
    pub launch_on_startup: &'static str,
    pub restore_on_exit: &'static str,
    pub language: &'static str,
    pub refresh: &'static str,
    pub quit: &'static str,
    pub show: &'static str,
    pub hide: &'static str,
    pub manual_placeholder: &'static str,
    pub settings_title: &'static str,
    pub no_targets: &'static str,
    pub target_count: &'static str,
    pub muted_count: &'static str,
    pub polling_interval: &'static str,
    pub unsupported_os: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_has_a_title() {
        for language in Language::ALL {
            assert!(!language.strings().app_title.is_empty());
            assert!(!language.native_name().is_empty());
        }
    }
}
