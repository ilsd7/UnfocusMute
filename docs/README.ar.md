<div align="center">
  <img src="../assets/app-icon.png" alt="أيقونة UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>تطبيق Windows خفيف ومحمول في علبة النظام لكتم التطبيقات المحددة عندما تصبح في الخلفية.<br>يعيد فقط الصوت الذي غيّره بنفسه.</strong></p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">تنزيل</a>
    · <a href="#الاستخدام">الاستخدام</a>
    · <a href="#الأمان-والخصوصية">الخصوصية</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=CI&logo=githubactions&logoColor=white" alt="CI status"></a>
    <img src="https://img.shields.io/badge/version-2.1.0-0D96F6?style=flat-square" alt="Version 2.1.0">
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    <img src="https://img.shields.io/badge/portable-yes-2E7D32?style=flat-square" alt="Portable app">
    <img src="https://img.shields.io/badge/Rust-native-B7410E?style=flat-square&logo=rust&logoColor=white" alt="Rust native app">
    <img src="https://img.shields.io/badge/binary-~491KB-5E35B1?style=flat-square" alt="Executable size about 491KB">
    <img src="https://img.shields.io/badge/telemetry-none-455A64?style=flat-square" alt="No telemetry">
  </p>

  <p>
    <a href="../README.md">한국어</a> · <a href="../README_en.md">English</a> · <a href="README.ja.md">日本語</a> · <a href="README.zh-CN.md">简体中文</a> · <a href="README.es.md">Español</a> · <a href="README.fr.md">Français</a> · <a href="README.pt.md">Português</a> · <a href="README.hi.md">हिन्दी</a> · العربية
  </p>
</div>

---

UnfocusMute هو تطبيق صغير وخفيف لعلبة نظام Windows. عندما تنتقل لعبة أو تطبيق قمت بتحديده إلى الخلفية، يكتم صوت ذلك التطبيق فقط تلقائيًا. لا يقتصر على الألعاب؛ يمكن أيضًا تسجيل المتصفحات، وتطبيقات المحادثة، والمشغلات، ومشغلات الوسائط، وأي تطبيق يظهر كجلسة صوت في Windows.

لأنه مبني كتطبيق Rust أصلي، يعمل مباشرة دون runtime منفصل. حجم ملف Windows التنفيذي الحالي نحو 491KB، أي أقل من 1MB.

<p align="center">
  <img src="../assets/screenshot_ar.png" alt="نافذة تطبيق UnfocusMute">
</p>

يكتم UnfocusMute جلسة الصوت الخاصة بالتطبيق المسجل فقط عندما لا يكون ذلك التطبيق في المقدمة. وعندما يعود التطبيق إلى المقدمة، يعيد UnfocusMute فقط الجلسات التي كتمها بنفسه، ولا يغيّر حالات الكتم التي عدّلتها يدويًا.

يكون ذلك مفيدًا خصوصًا عندما تبقي لعبة مفتوحة وتتنقل عبر Alt+Tab بين المتصفح أو تطبيق المحادثة أو نافذة العمل. يمكنك إيقاف صوت التطبيق في الخلفية فقط دون فتح خلاط مستوى الصوت في Windows مرارًا.

---

## التنزيل والتشغيل

على Windows 10/11، حمّل حزمة ZIP وفك ضغطها لتشغيل التطبيق مباشرة.

| أحدث حزمة |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [ملف تحقق SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [ملاحظات الإصدار](https://github.com/ilsd7/UnfocusMute/releases/latest) |

انقل مجلد `UnfocusMute-windows-x64` الناتج إلى المكان الذي تريد حفظ التطبيق فيه، ثم شغّل `UnfocusMute-<version>.exe` من داخل ذلك المجلد. التطبيق محمول، لذلك لا توجد عملية تثبيت ولا تحتاج إلى runtime منفصل أو Rust أو Visual Studio Build Tools أو MinGW.

## الاستخدام

1. شغّل UnfocusMute.
2. اختر اللغة من نافذة الاختيار عند التشغيل الأول. الاختيار الافتراضي هو English.
3. شغّل اللعبة أو التطبيق الذي تريد كتمه عندما يصبح في الخلفية.
4. افتح قائمة `ابحث عن عملية` أو اكتب كلمة بحث، ثم اختر عنصرًا واضغط `إضافة المحدد`.
5. إذا احتجت إلى تسجيل PID محدد فقط، اضغط `إظهار تفاصيل PID` واختر العنصر المنفصل. الأهداف المسجلة عبر PID تنطبق فقط على النسخة الحالية قيد التشغيل؛ إذا أُعيد تشغيل التطبيق وتغيّر PID، فاختره من جديد.
6. انقر بزر الفأرة الأيمن على تطبيق مسجل لتعديل ملاحظته أو `استبعاد من الكتم التلقائي` لذلك التطبيق.
7. عند إغلاق النافذة، يبقى التطبيق في علبة النظام ويواصل المراقبة. استخدم `خروج` لإنهائه بالكامل.

## استخدام ملاحظات التطبيقات المسجلة

إذا كان اسم العملية وحده لا يكفي لتذكّر التطبيق، فانقر بزر الفأرة الأيمن على التطبيق المسجل واختر `تعديل الملاحظة`. تظهر الملاحظة بجانب اسم العملية في قائمة التطبيقات المسجلة، ولا تؤثر في طريقة مطابقة الهدف.

هذا مفيد عندما يفتح مشغل لعبة واحد عدة عمليات، أو عندما لا يوضح اسم الملف التنفيذي وظيفته.

- `htgame.exe - NTE`
- `chrome.exe (PID 18432) - ملف شخصي لتشغيل الموسيقى`
- `game.exe (PID 21976) - عميل خادم الاختبار`
- `launcher.exe - المشغل قبل بدء اللعبة الفعلية`

تُحفظ الملاحظات محليًا مع بقية الإعدادات في `%APPDATA%\UnfocusMute\config.json`.

## معرفة اسم ملف تشغيل اللعبة

إذا لم تكن متأكدًا من الاسم الذي يجب تسجيله، فتحقق من اسم الملف التنفيذي الذي ينتهي بـ `.exe` في مدير المهام.

1. شغّل اللعبة أولًا.
2. استخدم `Alt`+`Tab` أو `Windows`+`Tab` لمغادرة شاشة اللعبة والعودة إلى Windows.
3. اضغط `Ctrl`+`Shift`+`Esc` لفتح مدير المهام.
4. رتّب قائمة العمليات حسب `CPU` للعثور على اللعبة التي شغّلتها للتو.
5. انقر بزر الفأرة الأيمن على اللعبة وافتح `خصائص`.
6. ابحث عن اسم الملف التنفيذي المنتهي بـ `.exe`، مثل `game.exe`، ثم أضفه إلى UnfocusMute.

## ملاحظات قبل الاستخدام

يعتمد UnfocusMute على أسماء العمليات ومعلومات النافذة الموجودة في المقدمة وجلسات CoreAudio التي يوفرها Windows. إذا لم ينشئ التطبيق جلسة صوت بعد، أو إذا حدّ برنامج تشغيل أو إعداد صلاحيات أو برنامج أمان من الوصول إلى الجلسة، فقد تكون القائمة أو التحكم في الكتم محدودين.

تستخدم أهداف PID أيضًا اسم `.exe` للنافذة الموجودة في المقدمة كحل احتياطي، لأن Windows لا يبلّغ دائمًا عن PID نفسه للنافذة النشطة وجلسة الصوت. إذا كانت عدة نسخ من نفس `.exe` تعمل في الوقت نفسه، فلا يمكن فصل هدف PID بينها بدقة كاملة: قد يعود الصوت عندما تكون نسخة أخرى من نفس `.exe` في المقدمة.

لا يحقن UnfocusMute أي كود داخل الألعاب، ولا يقرأ ذاكرة اللعبة، ولا يعترض الإدخال، ولا يغيّر ملفات اللعبة. يستخدم فقط معلومات العملية/النافذة في Windows ووظائف كتم جلسات CoreAudio، لذلك يُتوقع أن يعمل دون مشكلة مع معظم أنظمة مكافحة الغش، لكن لا يمكن ضمان التوافق مع كل نظام منها.

## الأمان والخصوصية

يعمل UnfocusMute بأسلوب محلي أولًا. يحفظ في ملف إعدادات محلي فقط أسماء العمليات المسجلة، وPID الاختيارية، ولغة الواجهة، وموضع النافذة، وخيارات بدء التشغيل.

تتم عملية اكتشاف جلسات الصوت والتحكم في الكتم داخل جهازك عبر واجهات Windows CoreAudio. لا توجد طلبات شبكة، أو حسابات، أو telemetry، أو أدوات تحليل، أو تقارير أعطال، أو تسجيل عن بُعد، ولا ينشئ UnfocusMute ملفات سجل منفصلة للتطبيق.

---

## طريقة العمل

يقارن UnfocusMute الأهداف المسجلة بالنافذة الموجودة حاليًا في المقدمة، ثم يكتم جلسة صوت التطبيق الهدف فقط عندما يكون ذلك التطبيق في الخلفية.

- إذا كان التطبيق الهدف في المقدمة، لا يغيّر حالة الصوت.
- إذا كان التطبيق الهدف في الخلفية، يكتم جلسة الصوت الخاصة بذلك التطبيق فقط.
- عندما يعود التطبيق الهدف إلى المقدمة، يعيد UnfocusMute فقط الجلسات التي كتمها بنفسه.
- حالات الكتم التي عدّلتها يدويًا في خلاط مستوى الصوت أو بأداة أخرى تبقى كما هي.

## مناسب عندما

- تترك لعبة أو تطبيقًا مفتوحًا وتتنقل كثيرًا إلى نافذة أخرى عبر Alt+Tab.
- لا يوفّر التطبيق أو اللعبة خيارًا داخليًا للكتم عند الانتقال إلى الخلفية.
- تريد كتم صوت اللعبة في الخلفية فقط مع إبقاء المتصفح أو تطبيق المكالمة مسموعًا.
- يفتح نفس `.exe` عدة عمليات وتحتاج إلى التبديل بين إدارة التطبيق كاملًا والتحكم في PID محدد.

## الميزات

- يكتم تلقائيًا جلسة الصوت للتطبيقات المسجلة عندما تكون في الخلفية ويعيدها عند عودتها إلى المقدمة.
- يضيف الأهداف من قائمة التطبيقات قيد التشغيل أو عبر كتابة اسم مثل `game.exe`.
- يدعم أهداف `.exe`، وأهداف PID للنسخة الحالية، وخيار `إظهار تفاصيل PID`.
- ملاحظات لكل تطبيق، و`استبعاد من الكتم التلقائي` و`إدراجه في الكتم التلقائي` لكل تطبيق.
- البقاء في علبة النظام، إيقاف مؤقت عام، فتح مجلد الإعدادات، ومنع تشغيل نسخ متعددة.
- اختيار اللغة عند التشغيل الأول، ثم التبديل داخل التطبيق بين English/한국어/日本語/简体中文/Español/Français/Português/हिन्दी/العربية.
- تُحفظ الإعدادات محليًا في `%APPDATA%\UnfocusMute\config.json`.

---

## البناء من المصدر

هدف الإصدار الموصى به هو `x86_64-pc-windows-msvc`.

المتطلبات:

- Rust stable
- Visual Studio Build Tools 2022 أو Visual Studio 2022
- Windows 10/11 SDK

```powershell
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked
```

إعدادات build الخاصة بالإصدار مضبوطة لتقليل حجم الملف الناتج. يستخدم release profile في `Cargo.toml` إزالة الرموز، وLTO، ووحدة codegen واحدة، و`panic = "abort"`، وتحسين الحجم.

الملف التنفيذي:

```text
target\x86_64-pc-windows-msvc\release\unfocusmute.exe
```

ملف ZIP للتوزيع:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

تحديث إشعارات تراخيص الطرف الثالث:

```powershell
cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md
```

يتم إنشاء الناتج في `dist\UnfocusMute-windows-x64.zip`، ومعه ملف تحقق SHA-256 في `dist\UnfocusMute-windows-x64.zip.sha256`. يحتوي ZIP على الملف التنفيذي الذي يتضمن رقم الإصدار (`UnfocusMute-<version>.exe`)، و`LICENSE`، و`THIRD_PARTY_NOTICES.md`، و`README_ko.txt` و`README_en.txt` في الجذر، بالإضافة إلى بقية ملفات README بصيغة `.txt` داخل مجلد `docs`.

---

## الترخيص

Apache License 2.0. راجع [LICENSE](../LICENSE) للتفاصيل.

راجع [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) لإشعارات تراخيص crates الخاصة بـ Rust من أطراف ثالثة.
