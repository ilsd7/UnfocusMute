# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | [Português](README.pt.md) | हिन्दी | [العربية](README.ar.md)

<p align="center">
  <img src="../assets/screenshot.png" alt="UnfocusMute ऐप विंडो" width="760">
</p>

UnfocusMute Windows ट्रे के लिए एक छोटा और हल्का ऐप है। जब चुना गया गेम या ऐप बैकग्राउंड में जाता है, तो यह सिर्फ उसी ऐप की आवाज अपने आप म्यूट करता है।

Rust native app के रूप में build होने के कारण यह अलग runtime के बिना सीधे चलता है। मौजूदा Windows executable लगभग 517 KB है, यानी 1 MB से कम।

जिन ऐप्स को संभालना है उन्हें रजिस्टर करें; जब वे सामने नहीं होते, UnfocusMute सिर्फ उनके ऑडियो सेशन म्यूट करता है। जब ऐप फिर से सामने आता है, तो UnfocusMute सिर्फ वही सेशन अनम्यूट करता है जिन्हें उसने खुद म्यूट किया था, इसलिए आपके मैन्युअल म्यूट वैसे ही रहते हैं।

जब आप Alt+Tab से गेम से ब्राउज़र, चैट ऐप या काम की विंडो पर जाते हैं, तब यह खास तौर पर उपयोगी है। बैकग्राउंड ऑडियो संभालने के लिए Windows वॉल्यूम मिक्सर बार-बार खोलने की जरूरत नहीं पड़ती।

## मुख्य फायदे

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

## Anti-Cheat संगतता

UnfocusMute games में code inject नहीं करता, game memory नहीं पढ़ता, input hook नहीं करता और game files modify नहीं करता। क्योंकि यह केवल Windows process/foreground window information और CoreAudio session mute controls इस्तेमाल करता है, इसलिए ज्यादातर anti-cheat systems में समस्या न होने की उम्मीद है, लेकिन हर anti-cheat system के साथ compatibility की guarantee नहीं दी जा सकती।

## Rust क्यों

UnfocusMute एक छोटा background utility है, इसलिए तेज start, कम memory use और आसान distribution मायने रखते हैं। Rust native executable अलग runtime के बिना चलता है और अनावश्यक resident framework जोड़े बिना Windows CoreAudio APIs से सीधे काम करता है।

## सुरक्षा और प्राइवेसी

UnfocusMute local-first तरीके से काम करता है। यह local config file में सिर्फ registered process names, optional PIDs, UI language, window position और startup preferences सेव करता है।

Audio session detection और mute control आपके PC पर Windows CoreAudio APIs के जरिए होते हैं। इसमें network requests, accounts, telemetry, analytics, crash reporting या remote logging नहीं है, और यह अलग app log files भी नहीं बनाता।

## डाउनलोड और चलाएं

Windows 10/11 ZIP डाउनलोड करें, extract करें और `UnfocusMute.exe` चलाएं। यह portable है, इसलिए installer की जरूरत नहीं है और अलग runtime, Rust, Visual Studio Build Tools या MinGW भी जरूरी नहीं हैं।

## उपयोग

1. UnfocusMute चलाएं।
2. पहली बार खुलने पर भाषा चुनें। डिफॉल्ट भाषा English है।
3. जिस गेम या ऐप को मैनेज करना है, उसे शुरू करें।
4. चल रहे ऐप्स की सूची refresh करें, कोई item चुनें और `चयनित जोड़ें` दबाएं।
5. एक जैसे `.exe` entry डिफॉल्ट रूप से साथ register होते हैं।
6. किसी खास process instance को register करना हो तो ही `PID विवरण दिखाएं` इस्तेमाल करें। PID target सिर्फ अभी चल रहे instance पर लागू होता है; app restart होकर PID बदल जाए तो उसे फिर से चुनें।
7. Window बंद करने पर UnfocusMute tray में चलता रहता है। पूरी तरह बंद करने के लिए `बंद करें` इस्तेमाल करें।

## डिफॉल्ट सेटिंग

पहली बार चलाने पर आप चुन सकते हैं कि Windows में sign in करते ही UnfocusMute अपने आप शुरू हो या नहीं। नई config में auto-start बंद रहता है, start minimized to tray चालू रहता है, और exit पर apps को unmute करना चालू रहता है।

ऐप में `भाषा` पर क्लिक करके English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी और العربية के बीच तुरंत बदल सकते हैं। चुनी गई भाषा अपने आप सेव हो जाती है।

## डेवलपर बिल्ड

Recommended release target `x86_64-pc-windows-msvc` है।

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

Release builds छोटे output size के लिए configured हैं। `Cargo.toml` का release profile symbols हटाता है, LTO enable करता है, एक codegen unit इस्तेमाल करता है, `panic = "abort"` सेट करता है और size के लिए optimize करता है।

Distribution ZIP बनाएं:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

पैकेज `dist\UnfocusMute-<version>-windows-x64.zip` में बनता है और इसमें executable फ़ाइल, `LICENSE`, `THIRD_PARTY_NOTICES.md`, root में `README_ko.md` और `README_en.md`, और `docs` में बाकी स्थानीयकृत दस्तावेज़ शामिल होते हैं।

## लाइसेंस

Apache License 2.0. [LICENSE](../LICENSE) देखें।

Third-party Rust crate license notices के लिए [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) देखें।
