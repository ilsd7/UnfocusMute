# UnfocusMute

[한국어](../README.md) | [English](../README_en.md) | [日本語](README.ja.md) | [简体中文](README.zh-CN.md) | [Español](README.es.md) | [Français](README.fr.md) | [Português](README.pt.md) | हिन्दी | [العربية](README.ar.md)

UnfocusMute Windows ट्रे के लिए एक छोटा और हल्का ऐप है। जब चुने गए गेम या ऐप बैकग्राउंड में चले जाते हैं, तो यह उन्हें अपने आप म्यूट कर देता है।

यह Rust से बना पोर्टेबल ऐप है और केवल उन्हीं ऑडियो सेशन को नियंत्रित करता है जिन्हें आप रजिस्टर करते हैं। जब ऐप फिर से सामने आता है, तो UnfocusMute सिर्फ वही सेशन अनम्यूट करता है जिन्हें उसने खुद म्यूट किया था, इसलिए आपके मैन्युअल म्यूट नहीं बदलते।

जब आप Alt+Tab से गेम से ब्राउज़र, चैट ऐप या काम की विंडो पर जाते हैं, तब यह खास तौर पर उपयोगी है। किसी एक बैकग्राउंड ऐप को शांत रखने के लिए Windows वॉल्यूम मिक्सर बार-बार खोलने की जरूरत नहीं पड़ती।

## मुख्य फायदे

- `.exe` स्तर और अलग-अलग PID, दोनों तरह से लक्ष्य रजिस्टर कर सकते हैं।
- एक ही executable से चल रहे कई PID को साथ मैनेज कर सकते हैं।
- जरूरत होने पर `PID विवरण दिखाएं` से सिर्फ एक खास process instance रजिस्टर कर सकते हैं।
- UnfocusMute केवल उन्हीं सेशन को वापस अनम्यूट करता है जिन्हें उसने खुद म्यूट किया था।
- ZIP से सीधे चलने वाला portable app है, installer की जरूरत नहीं।

## कब उपयोगी है

- गेम या ऐप से Alt+Tab करके दूसरी विंडो पर बार-बार जाते समय
- ऐसे गेम जिनमें बैकग्राउंड म्यूट की जरूरत होती है
- ऐसे गेम जिनमें बैकग्राउंड में जाने पर म्यूट करने का अपना विकल्प नहीं है
- बैकग्राउंड गेम की आवाज बंद रखनी हो, लेकिन ब्राउज़र या कॉल ऐप सुनाई देता रहे
- किसी ऐप को `.exe` से मैनेज करना हो या सिर्फ एक PID नियंत्रित करना हो

## मुख्य सुविधाएं

- केवल उन रजिस्टर किए गए ऐप्स को अपने आप म्यूट करता है जो फोकस में नहीं हैं।
- केवल UnfocusMute द्वारा म्यूट किए गए सेशन ही वापस अनम्यूट करता है।
- चल रहे ऐप्स की सूची से लक्ष्य जोड़ें या `game.exe` जैसा executable नाम टाइप करें।
- `.exe` group registration और per-PID registration, दोनों सपोर्ट करता है।
- ट्रे में चलता है, pause, config file shortcut और single-instance protection देता है।
- पहली बार भाषा चुनने और ऐप में तुरंत English, 한국어, 日本語, 简体中文, Español, Français, Português, हिन्दी और العربية में बदलने की सुविधा।
- लोकल config `%APPDATA%\UnfocusMute\config.json` में सेव होती है।

## Rust क्यों

UnfocusMute एक छोटा background utility है, इसलिए तेज start, कम memory use और आसान distribution मायने रखते हैं। Rust native executable अलग runtime के बिना चलता है और अनावश्यक resident framework जोड़े बिना Windows CoreAudio APIs से सीधे काम करता है।

## सुरक्षा और प्राइवेसी

UnfocusMute local-first तरीके से काम करता है। यह local config file में सिर्फ registered process names, optional PIDs, UI language, window position और startup preferences सेव करता है।

Audio session detection और mute control आपके PC पर Windows CoreAudio APIs के जरिए होते हैं। इसमें network requests, accounts, telemetry, analytics, crash reporting, remote logging या अलग app log files नहीं हैं।

## डाउनलोड और चलाएं

Windows 10/11 ZIP डाउनलोड करें, extract करें और `UnfocusMute.exe` चलाएं। यह portable है, इसलिए installer की जरूरत नहीं है और अलग runtime, Rust, Visual Studio Build Tools या MinGW भी जरूरी नहीं हैं।

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

पैकेज `dist\UnfocusMute-<version>-windows-x64.zip` में बनता है और इसमें executable फ़ाइल, `LICENSE`, root में `README_ko.md` और `README_en.md`, और `docs` में बाकी स्थानीयकृत दस्तावेज़ शामिल होते हैं।

## लाइसेंस

Apache License 2.0. [LICENSE](../LICENSE) देखें।
