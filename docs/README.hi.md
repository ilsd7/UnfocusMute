# UnfocusMute

[한국어](../README.md) | [English](README.en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | [Português](README.pt.md) | हिन्दी | [العربية](README.ar.md)

UnfocusMute Windows ट्रे के लिए एक छोटा और हल्का ऐप है। जब चुने गए गेम या ऐप बैकग्राउंड में चले जाते हैं, तो यह उन्हें अपने आप म्यूट कर देता है।

यह Rust से बना पोर्टेबल ऐप है और केवल उन्हीं ऑडियो सेशन को नियंत्रित करता है जिन्हें आप रजिस्टर करते हैं। जब ऐप फिर से सामने आता है, तो UnfocusMute सिर्फ वही सेशन अनम्यूट करता है जिन्हें उसने खुद म्यूट किया था, इसलिए आपके मैन्युअल म्यूट नहीं बदलते।

## कब उपयोगी है

- गेम से ब्राउज़र, चैट ऐप या काम की विंडो पर बार-बार स्विच करते समय
- Windows वॉल्यूम मिक्सर खोले बिना किसी बैकग्राउंड ऐप को शांत रखना हो
- ऐसे ऐप मैनेज करने हों जिनमें एक ही `.exe` के कई प्रोसेस चलते हैं, जैसे ब्राउज़र
- ZIP से सीधे चलने वाला हल्का Rust-आधारित टूल चाहिए, बिना इंस्टॉलर

## मुख्य सुविधाएं

- केवल उन रजिस्टर किए गए ऐप्स को अपने आप म्यूट करता है जो फोकस में नहीं हैं
- केवल UnfocusMute द्वारा म्यूट किए गए सेशन ही वापस अनम्यूट करता है
- चल रहे ऐप्स की सूची से लक्ष्य जोड़ें या `game.exe` जैसा executable नाम टाइप करें
- एक जैसे `.exe` प्रोसेस को डिफॉल्ट रूप से साथ रखता है, जो कई PID वाले ब्राउज़र के लिए उपयोगी है
- `PID विवरण दिखाएं` से किसी खास PID को रजिस्टर कर सकते हैं
- ट्रे में चलता है, pause, config file shortcut और single-instance protection देता है
- पहली बार भाषा चुनने और ऐप में तुरंत English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी और العربية में बदलने की सुविधा
- लोकल config `%APPDATA%\UnfocusMute\config.json` में सेव होती है
- कोई network request, account, telemetry या अलग app log नहीं

## डाउनलोड और चलाएं

Windows ZIP डाउनलोड करें, extract करें और `UnfocusMute.exe` चलाएं। यह portable है, इसलिए installer की जरूरत नहीं है और अलग runtime, Rust, Visual Studio Build Tools या MinGW भी जरूरी नहीं हैं।

## उपयोग

1. UnfocusMute चलाएं।
2. पहली बार खुलने पर भाषा चुनें। डिफॉल्ट भाषा English है।
3. जिस गेम या ऐप को मैनेज करना है, उसे शुरू करें।
4. चल रहे ऐप्स की सूची refresh करें, कोई item चुनें और `चयनित जोड़ें` दबाएं।
5. एक जैसे `.exe` entry डिफॉल्ट रूप से साथ register होते हैं।
6. किसी खास process instance को register करना हो तो ही `PID विवरण दिखाएं` इस्तेमाल करें।
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

Distribution ZIP बनाएं:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Package `dist\UnfocusMute-<version>-windows-x64.zip` में बनता है और इसमें executable और `LICENSE` शामिल होते हैं।

## प्राइवेसी

UnfocusMute local config file में सिर्फ registered process names, optional PIDs, UI language, window position और startup preferences सेव करता है। Audio session detection और mute control Windows CoreAudio APIs के जरिए local machine पर ही किए जाते हैं।

## लाइसेंस

Apache License 2.0. [LICENSE](../LICENSE) देखें।
