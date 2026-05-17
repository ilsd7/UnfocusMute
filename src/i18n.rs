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
                app_subtitle: "게임이나 앱이 백그라운드로 전환되면 선택한 오디오만 자동으로 뮤트합니다.",
                registered_processes: "등록된 앱",
                running_processes: "실행 중인 앱",
                add_process_section: "새 대상 추가",
                running_process_hint: "기본 목록은 같은 .exe를 하나로 묶습니다.",
                show_pid_details: "세부 PID 보기",
                hide_pid_details: "exe별 보기",
                manual_process: "프로세스 이름",
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
                no_targets: "등록된 앱이 없습니다. 오른쪽에서 실행 중인 앱을 추가하세요.",
                target_count: "등록 앱",
                muted_count: "현재 음소거",
                open_config: "설정 파일 열기",
                unsupported_os: "UnfocusMute는 Windows 11용 앱입니다. 이 환경에서는 코어 테스트와 문서 작업만 실행할 수 있습니다.",
            },
            Language::En => Strings {
                app_title: "UnfocusMute",
                status_running: "Watching",
                status_paused: "Paused",
                app_subtitle: "Automatically mutes selected app audio only while the app is in the background.",
                registered_processes: "Registered apps",
                running_processes: "Running apps",
                add_process_section: "Add target",
                running_process_hint: "The default list groups duplicate .exe processes.",
                show_pid_details: "Show PIDs",
                hide_pid_details: "Group by exe",
                manual_process: "Process name",
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
                no_targets: "No apps are registered. Add a running app on the right.",
                target_count: "Registered apps",
                muted_count: "Currently muted",
                open_config: "Open config file",
                unsupported_os: "UnfocusMute is a Windows 11 app. This environment can run only core tests and documentation tasks.",
            },
            Language::Ja => Strings {
                app_title: "UnfocusMute",
                status_running: "監視中",
                status_paused: "一時停止",
                app_subtitle: "対象アプリがバックグラウンドの間だけ、そのオーディオを自動でミュートします。",
                registered_processes: "登録済みアプリ",
                running_processes: "実行中のアプリ",
                add_process_section: "対象を追加",
                running_process_hint: "既定では同じ.exeプロセスを1つにまとめます。",
                show_pid_details: "PID詳細を表示",
                hide_pid_details: "exe別に表示",
                manual_process: "プロセス名",
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
                no_targets: "登録済みアプリはありません。右側から実行中のアプリを追加してください。",
                target_count: "登録アプリ",
                muted_count: "現在ミュート",
                open_config: "設定ファイルを開く",
                unsupported_os: "UnfocusMuteはWindows 11向けアプリです。この環境ではコアテストとドキュメント作業のみ実行できます。",
            },
            Language::ZhHans => Strings {
                app_title: "UnfocusMute",
                status_running: "监视中",
                status_paused: "已暂停",
                app_subtitle: "当目标应用进入后台时，仅自动静音所选应用的音频。",
                registered_processes: "已注册应用",
                running_processes: "正在运行的应用",
                add_process_section: "添加目标",
                running_process_hint: "默认列表会合并相同的 .exe 进程。",
                show_pid_details: "显示 PID",
                hide_pid_details: "按 exe 分组",
                manual_process: "进程名称",
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
                no_targets: "尚未注册任何应用。请从右侧添加正在运行的应用。",
                target_count: "已注册应用",
                muted_count: "当前静音",
                open_config: "打开配置文件",
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
    pub app_subtitle: &'static str,
    pub registered_processes: &'static str,
    pub running_processes: &'static str,
    pub add_process_section: &'static str,
    pub running_process_hint: &'static str,
    pub show_pid_details: &'static str,
    pub hide_pid_details: &'static str,
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
    pub open_config: &'static str,
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
