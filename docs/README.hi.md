<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMute आइकन" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>चुने हुए background apps को mute करने वाला हल्का, portable Windows tray app।<br>यह सिर्फ वही audio restore करता है जिसे इसने खुद बदला था।</strong></p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · हिन्दी · <a href="README.ar.md">العربية</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/version-2.1.1-0D96F6?style=flat-square" alt="Version 2.1.1">
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <img src="https://img.shields.io/badge/portable-yes-2E7D32?style=flat-square" alt="Portable app">
    <img src="https://img.shields.io/badge/Rust-native-B7410E?style=flat-square&logo=rust&logoColor=white" alt="Rust native app">
    <img src="https://img.shields.io/badge/binary-~491KB-5E35B1?style=flat-square" alt="Executable size about 491KB">
    <img src="https://img.shields.io/badge/telemetry-none-455A64?style=flat-square" alt="No telemetry">
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">डाउनलोड</a>
    · <a href="#उपयोग">उपयोग</a>
    · <a href="#सुरक्षा-और-प्राइवेसी">प्राइवेसी</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

---

UnfocusMute Windows tray के लिए एक छोटा और हल्का app है। चुना हुआ game या app background में जाते ही यह सिर्फ उसी app की आवाज अपने आप mute करता है। यह सिर्फ games तक सीमित नहीं है: browsers, messengers, launchers, media players और दूसरे apps भी register किए जा सकते हैं, अगर वे Windows audio session के रूप में दिखते हों।

Rust native app के रूप में build होने के कारण यह अलग runtime के बिना सीधे चलता है। मौजूदा Windows executable लगभग 491 KB है, यानी 1 MB से कम।

<p align="center">
  <img src="../assets/screenshot_hi.png" alt="UnfocusMute ऐप विंडो">
</p>

UnfocusMute registered app की audio session को सिर्फ तब mute करता है जब वह app foreground में नहीं होता। जब app वापस foreground में आता है, तो UnfocusMute सिर्फ वही sessions restore करता है जिन्हें उसने खुद mute किया था। जिन्हें UnfocusMute ने mute नहीं किया था, या जिन्हें आपने पहले से mute कर रखा था, उन्हें यह unmute नहीं करता।

जब कोई game खुला हो और आप Alt+Tab से browser, chat app या work window में जाते-आते हों, तब यह खास तौर पर उपयोगी है। Windows volume mixer बार-बार खोले बिना सिर्फ background app की आवाज बंद की जा सकती है।

---

## डाउनलोड और चलाएं

Windows 10/11 पर ZIP package डाउनलोड करके extract करें, फिर app चला सकते हैं।

| नवीनतम पैकेज |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [SHA-256 file](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [रिलीज़ नोट्स](https://github.com/ilsd7/UnfocusMute/releases/latest) |

निकले हुए `UnfocusMute-windows-x64` folder को अपनी पसंद की जगह पर ले जाएं, फिर उसी folder में `UnfocusMute-<version>.exe` चलाएं। यह portable app है, इसलिए installer की जरूरत नहीं है और अलग runtime, Rust, Visual Studio Build Tools या MinGW भी जरूरी नहीं हैं।

## उपयोग

1. UnfocusMute चलाएं।
2. पहली बार खुलने वाली भाषा स्क्रीन में भाषा चुनें। Default चयन English है।
3. वह game या app शुरू करें जिसे background में जाने पर mute करना है।
4. `प्रक्रिया खोजें` list खोलें या search term लिखें, item चुनें और `चयनित जोड़ें` दबाएं।
5. अगर सिर्फ किसी खास PID को register करना है, तो `PID विवरण दिखाएं` दबाकर अलग entry चुनें। PID target सिर्फ अभी चल रहे instance पर लागू होता है; app restart होकर PID बदल जाए तो उसे फिर से चुनें।
6. पंजीकृत ऐप पर right-click करके उसका नोट संपादित किया जा सकता है या उस ऐप को `ऑटो-म्यूट से बाहर करें` किया जा सकता है।
7. विंडो बंद करने पर ऐप tray में रहकर निगरानी जारी रखता है। पूरी तरह बंद करने के लिए `बंद करें` दबाएं।

## पंजीकृत ऐप्स में नोट इस्तेमाल करें

अगर प्रक्रिया नाम देखकर ऐप पहचानना मुश्किल हो, तो पंजीकृत ऐप पर right-click करें और `नोट संपादित करें` चुनें। नोट पंजीकृत ऐप सूची में प्रक्रिया नाम के साथ दिखाई देता है और लक्ष्य पहचान को प्रभावित नहीं करता।

यह तब काम आता है जब एक ही game launcher कई प्रक्रियाएं खोलता हो, या executable name देखकर उसका उपयोग साफ न समझ आए।

- `htgame.exe - NTE`
- `chrome.exe (PID 18432) - music playback profile`
- `game.exe (PID 21976) - test server client`
- `launcher.exe - असली game शुरू होने से पहले वाला launcher`

नोट बाकी settings के साथ local रूप से `%APPDATA%\UnfocusMute\config.json` में save होते हैं।

## गेम का executable नाम कैसे देखें

अगर register करने वाला नाम साफ न हो, तो Task Manager में `.exe` पर खत्म होने वाला executable name देखें।

1. पहले game शुरू करें।
2. `Alt`+`Tab` या `Windows`+`Tab` से game screen छोड़कर Windows पर लौटें।
3. Task Manager खोलने के लिए `Ctrl`+`Shift`+`Esc` दबाएं।
4. Process list को `CPU` के अनुसार sort करके अभी शुरू किया हुआ game ढूंढें।
5. Game item पर right-click करें और `Properties` खोलें।
6. `game.exe` जैसा `.exe` पर खत्म होने वाला executable name देखें और उसे UnfocusMute में add करें।

## इस्तेमाल से पहले ध्यान दें

UnfocusMute Windows से मिलने वाले process names, foreground window information और CoreAudio sessions के आधार पर काम करता है। अगर किसी app ने अभी audio session नहीं बनाया है, या driver, permission setting या security tool session access को सीमित करता है, तो list दिखना या mute control सीमित हो सकता है।

PID targets foreground window के `.exe` name को fallback की तरह भी इस्तेमाल करते हैं, क्योंकि Windows foreground window और audio session के लिए हमेशा एक ही PID नहीं बताता। अगर एक ही `.exe` के कई instances चल रहे हों, तो PID target उन्हें पूरी तरह अलग नहीं कर सकता: उसी `.exe` का दूसरा instance foreground में आने पर आवाज़ वापस आ सकती है।

UnfocusMute games में code inject नहीं करता, game memory नहीं पढ़ता, input hook नहीं करता और game files modify नहीं करता। यह सिर्फ Windows process/foreground window information और CoreAudio session mute controls इस्तेमाल करता है, इसलिए ज्यादातर anti-cheat systems में समस्या नहीं होनी चाहिए, लेकिन हर anti-cheat के साथ compatibility की guarantee नहीं दी जा सकती।

## सुरक्षा और प्राइवेसी

UnfocusMute local-first तरीके से काम करता है। यह local config file में सिर्फ registered process names, optional PIDs, UI language, window position और startup options save करता है।

Audio session detection और mute control आपके PC पर Windows CoreAudio APIs के जरिए ही होते हैं। इसमें network requests, accounts, telemetry, analytics, crash reporting या remote logging नहीं है, और UnfocusMute अलग app log files भी नहीं बनाता।

---

## यह कैसे काम करता है

UnfocusMute registered targets और अभी foreground में मौजूद window की तुलना करता है, और target app background में होने पर ही उसकी audio session mute करता है।

- Target app foreground में हो तो audio state नहीं बदली जाती।
- Target app background में हो तो सिर्फ उसी app की audio session mute होती है।
- Target app फिर foreground में आए तो UnfocusMute सिर्फ अपने द्वारा mute की गई sessions restore करता है।
- जिन्हें UnfocusMute ने mute नहीं किया था, या जिन्हें आपने पहले से mute कर रखा था, उन्हें UnfocusMute unmute नहीं करता।

## कब उपयोगी है

- Game या app खुला रखते हुए आप अक्सर Alt+Tab से दूसरी window में जाते हों।
- Game या app में background में जाने पर mute करने का अपना option न हो।
- Browser या call app की आवाज रखते हुए सिर्फ background game audio बंद करना हो।
- एक ही `.exe` कई processes खोलता हो और कभी पूरे app को, कभी किसी खास PID को control करना हो।

## मुख्य सुविधाएं

- Registered apps background में होने पर audio session अपने आप mute करता है और foreground में लौटने पर restore करता है।
- Running app list से target जोड़ें या `game.exe` जैसा नाम manually लिखें।
- `.exe` targets, current-instance PID targets और `PID विवरण दिखाएं` support करता है।
- App-wise notes, app-wise `ऑटो-म्यूट से बाहर करें` और `ऑटो-म्यूट में शामिल करें`।
- Tray में चलना, global pause, config folder खोलना और duplicate instance protection।
- पहली बार language चुनें, फिर app के अंदर English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية तुरंत बदलें।
- Settings local रूप से `%APPDATA%\UnfocusMute\config.json` में save होती हैं।

---

## सोर्स से बिल्ड करें

Recommended release target `x86_64-pc-windows-msvc` है।

Requirements:

- Rust stable
- Visual Studio Build Tools 2022 या Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

Release build छोटे output size के लिए configured है। `Cargo.toml` का release profile symbols strip करता है, LTO enable करता है, single codegen unit इस्तेमाल करता है, `panic = "abort"` set करता है और size के लिए optimize करता है।

Executable:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

Distribution ZIP:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Third-party license notices refresh करें:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

परिणाम `dist\UnfocusMute-windows-x64.zip` में बनता है, और उसी जगह SHA-256 जांच फ़ाइल `dist\UnfocusMute-windows-x64.zip.sha256` भी बनती है। ZIP में version वाला executable (`UnfocusMute-<version>.exe`), `LICENSE`, `THIRD_PARTY_NOTICES.md`, root के `README_ko.txt` और `README_en.txt`, और `docs` folder की बाकी README `.txt` files शामिल होती हैं।

---

## लाइसेंस

Apache License 2.0। जानकारी के लिए [LICENSE](../LICENSE) देखें।

तीसरे पक्ष के Rust crate license notices के लिए [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) देखें।
