<div align="center">
  <img src="../assets/app-icon.png" alt="UnfocusMute आइकन" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>Windows tray के लिए हल्का portable app, जो selected background apps को mute करता है और सिर्फ वही audio restore करता है जिसे उसने बदला था।</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases">डाउनलोड</a>
    · <a href="#उपयोग">उपयोग</a>
    · <a href="#सुरक्षा-और-प्राइवेसी">प्राइवेसी</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>

  <p>
    <code>Windows 10/11</code>
    <code>Portable</code>
    <code>Rust Native</code>
    <code>~477KB</code>
    <code>No telemetry</code>
  </p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · हिन्दी · <a href="README.ar.md">العربية</a>
  </p>
</div>

UnfocusMute Windows ट्रे के लिए एक छोटा और हल्का ऐप है। जब चुना गया गेम या ऐप बैकग्राउंड में जाता है, तो यह सिर्फ उसी ऐप की आवाज अपने आप म्यूट करता है।

Rust native app के रूप में build होने के कारण यह अलग runtime के बिना सीधे चलता है। मौजूदा Windows executable लगभग 477 KB है, यानी 1 MB से कम।

<p align="center">
  <img src="../assets/screenshot.png" alt="UnfocusMute ऐप विंडो" width="760" loading="lazy" decoding="async">
</p>

जिन ऐप्स को संभालना है उन्हें रजिस्टर करें; जब वे सामने नहीं होते, UnfocusMute सिर्फ उनके ऑडियो सेशन म्यूट करता है। जब ऐप फिर से सामने आता है, तो UnfocusMute सिर्फ वही सेशन अनम्यूट करता है जिन्हें उसने खुद म्यूट किया था, इसलिए आपके मैन्युअल म्यूट वैसे ही रहते हैं।

जब आप Alt+Tab से गेम से ब्राउज़र, चैट ऐप या काम की विंडो पर जाते हैं, तब यह खास तौर पर उपयोगी है। बैकग्राउंड ऑडियो संभालने के लिए Windows वॉल्यूम मिक्सर बार-बार खोलने की जरूरत नहीं पड़ती।

---

## डाउनलोड और चलाएं

[GitHub Releases](https://github.com/ilsd7/UnfocusMute/releases) से Windows 10/11 ZIP डाउनलोड करें, extract करें और `UnfocusMute.exe` चलाएं। यह portable है, इसलिए installer की जरूरत नहीं है और अलग runtime, Rust, Visual Studio Build Tools या MinGW भी जरूरी नहीं हैं।

## उपयोग

1. UnfocusMute चलाएं।
2. पहली बार खुलने पर भाषा चुनें। डिफॉल्ट भाषा English है।
3. जिस गेम या ऐप को मैनेज करना है, उसे शुरू करें।
4. चल रहे ऐप्स की सूची refresh करें, कोई item चुनें और `चयनित जोड़ें` दबाएं।
5. किसी खास process instance को register करना हो तो ही `PID विवरण दिखाएं` इस्तेमाल करें। PID से register किया गया target सिर्फ अभी चल रहे instance पर लागू होता है; app restart होकर PID बदल जाए तो उसे फिर से चुनें।
6. Window बंद करने पर UnfocusMute tray में चलता रहता है। पूरी तरह बंद करने के लिए `बंद करें` इस्तेमाल करें।

## गेम का executable नाम कैसे देखें

अगर समझ न आए कि कौन सा नाम लिखना है, तो Task Manager में `.exe` पर खत्म होने वाला executable नाम देखें।

1. पहले गेम चलाएं।
2. `Alt`+`Tab` या `Windows`+`Tab` से गेम screen से बाहर निकलकर Windows पर लौटें।
3. `Ctrl`+`Shift`+`Esc` दबाकर Task Manager खोलें।
4. Process list को `CPU` के हिसाब से sort करें ताकि अभी चलाया गया गेम जल्दी मिले।
5. गेम पर right-click करें और `Properties` खोलें।
6. `game.exe` जैसे `.exe` पर खत्म होने वाले executable नाम को देखें और UnfocusMute में add करें।

---

## फायदे

- गेम या ऐप के हिसाब से बैकग्राउंड म्यूट अपने आप होता है, इसलिए window बदलते समय आवाज अलग से संभालनी नहीं पड़ती।
- UnfocusMute सिर्फ वही ऑडियो वापस अनम्यूट करता है जिसे उसने बदला था, मैन्युअल म्यूट वैसे ही रहते हैं।
- बिना installer, account या network connection के local चलने वाला हल्का portable tool है।

## कब उपयोगी है

- गेम या ऐप से Alt+Tab करके दूसरी विंडो पर बार-बार जाते समय
- ऐसे गेम जिनमें बैकग्राउंड में जाने पर म्यूट करने का अपना विकल्प नहीं है
- बैकग्राउंड गेम की आवाज बंद रखनी हो, लेकिन ब्राउज़र या कॉल ऐप सुनाई देता रहे
- एक ही `.exe` से कई processes खुलते हों और कभी पूरे app को, कभी सिर्फ एक PID को control करना हो

## मुख्य सुविधाएं

- रजिस्टर किए गए ऐप्स बैकग्राउंड में हों तो उनके audio sessions अपने आप mute करता है, और foreground में लौटने पर restore करता है।
- चल रहे ऐप्स की सूची से target जोड़ें या `game.exe` जैसा executable नाम टाइप करें।
- `.exe` target, current instance के PID target और `PID विवरण दिखाएं` को support करता है।
- tray में चलता है और pause, config folder खोलने व single-instance protection की सुविधा देता है।
- पहली बार भाषा चुनें, फिर app में तुरंत English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी और العربية के बीच बदलें।
- लोकल config `%APPDATA%\UnfocusMute\config.json` में सेव होती है।

---

## इस्तेमाल से पहले ध्यान दें

UnfocusMute games में code inject नहीं करता, game memory नहीं पढ़ता, input hook नहीं करता और game files modify नहीं करता। क्योंकि यह केवल Windows process/foreground window information और CoreAudio session mute controls इस्तेमाल करता है, इसलिए ज्यादातर anti-cheat systems में समस्या न होने की उम्मीद है, लेकिन हर anti-cheat system के साथ compatibility की guarantee नहीं दी जा सकती।

## सुरक्षा और प्राइवेसी

UnfocusMute local-first तरीके से काम करता है। यह local config file में सिर्फ registered process names, optional PIDs, UI language, window position और startup preferences सेव करता है।

Audio session detection और mute control आपके PC पर Windows CoreAudio APIs के जरिए होते हैं। इसमें network requests, accounts, telemetry, analytics, crash reporting या remote logging नहीं है, और यह अलग app log files भी नहीं बनाता।

## शुरुआती सेटिंग

पहली बार चलाने पर आप चुन सकते हैं कि Windows में sign in करते ही UnfocusMute अपने आप शुरू हो या नहीं। नई config में auto-start बंद रहता है, start minimized to tray चालू रहता है, और exit पर apps को unmute करना चालू रहता है।

ऐप में `भाषा` पर क्लिक करके English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी और العربية के बीच तुरंत बदल सकते हैं। चुनी गई भाषा अपने आप सेव हो जाती है।

Config file सीधे देखनी हो या backup files manage करनी हों, तो app में `Config folder खोलें` इस्तेमाल करें।

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

Release builds छोटे output size के लिए configured हैं। `Cargo.toml` का release profile symbols हटाता है, LTO enable करता है, एक codegen unit इस्तेमाल करता है, `panic = "abort"` सेट करता है और size के लिए optimize करता है।

Executable:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

Distribution ZIP बनाएं:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Third-party license notices refresh करें:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

पैकेज `dist\UnfocusMute-<version>-windows-x64.zip` में बनता है, और SHA-256 सत्यापन फ़ाइल `dist\UnfocusMute-<version>-windows-x64.zip.sha256` में बनती है। ZIP में executable फ़ाइल, `LICENSE`, `THIRD_PARTY_NOTICES.md`, root में `README_ko.md` और `README_en.md`, और `docs` में बाकी स्थानीयकृत दस्तावेज़ शामिल होते हैं।

---

## लाइसेंस

Apache License 2.0. [LICENSE](../LICENSE) देखें।

Third-party Rust crate license notices के लिए [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) देखें।
