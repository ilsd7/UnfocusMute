<div align="center" dir="rtl">
  <img src="../assets/app-icon.png" alt="أيقونة UnfocusMute" width="96" height="96">

  <h1>UnfocusMute</h1>

  <p><strong>تطبيق Windows خفيف يعمل من علبة النظام، يكتم تلقائيًا الألعاب والتطبيقات المختارة عندما تفقد نافذتها التركيز.</strong></p>

  <p dir="ltr">
    <a href="../README.md">English</a> · <a href="README_ko.md">한국어</a> · <a href="README_ja.md">日本語</a> · <a href="README_zh-CN.md">简体中文</a> · <a href="README_es.md">Español</a> · <a href="README_fr.md">Français</a> · <a href="README_pt.md">Português</a> · <a href="README_hi.md">हिन्दी</a> · العربية
  </p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/ilsd7/UnfocusMute/ci.yml?branch=main&style=flat-square&label=Build&logo=githubactions&logoColor=white" alt="Build status"></a>
    &nbsp;
    <img src="https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows 10/11">
    &nbsp;
    <a href="../LICENSE"><img src="https://img.shields.io/badge/License-Apache--2.0-blue?style=flat-square" alt="Apache-2.0 license"></a>
  </p>

  <p>محلي بالكامل &nbsp;·&nbsp; بلا وصول إلى الشبكة &nbsp;·&nbsp; بلا قياس عن بُعد &nbsp;·&nbsp; بلا تثبيت</p>

  <p>
    <a href="https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip">تنزيل</a>
    · <a href="#الاستخدام">الاستخدام</a>
    · <a href="#الأمان-والخصوصية">الخصوصية</a>
    · <a href="../LICENSE">Apache-2.0</a>
  </p>
</div>

<br>

<div dir="rtl">

UnfocusMute هو تطبيق Windows صغير وخفيف يعمل من علبة النظام، يكتم تلقائيًا الألعاب والتطبيقات المختارة عندما تنتقل إلى الخلفية، ثم يستعيد صوتها عندما تعود إلى المقدمة.

- مبني كتطبيق أصلي بلغة Rust ويعمل مباشرة دون بيئة تشغيل منفصلة.
- يبلغ حجم الملف التنفيذي نحو 500 كيلوبايت.
- لا يقتصر على الألعاب؛ يمكنك أيضًا تسجيل تطبيقات عادية مثل المتصفحات، وتطبيقات المراسلة، ومشغلات الألعاب، ومشغلات الوسائط.
- لا يطبّق كتم الصوت واستعادته إلا على الجلسات الصوتية التي غيّر UnfocusMute حالة كتمها بنفسه؛ أما الجلسات التي كتمتها أنت مسبقًا فلا يلمسها.

<p align="center">
  <img src="../assets/screenshot_ar.png" width="600" alt="نافذة UnfocusMute الرئيسية">
</p>

<br>

## مفيد في الحالات التالية

- تترك لعبة أو تطبيقًا مفتوحًا وتتنقل كثيرًا بين النوافذ باستخدام Alt+Tab.
- تريد إبقاء تطبيق ما صامتًا لأنه لا يوفّر خيارًا للكتم في الخلفية.
- تريد كتم صوت تطبيق محدد فقط عندما لا يكون في المقدمة أثناء العمل على مهام أخرى.

## التنزيل والتشغيل

على Windows 10/11، حمّل حزمة ZIP وفك ضغطها لتشغيل التطبيق مباشرة.

| أحدث حزمة |
| --- |
| [UnfocusMute-windows-x64.zip](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip) |
| [ملف تحقق SHA-256](https://github.com/ilsd7/UnfocusMute/releases/latest/download/UnfocusMute-windows-x64.zip.sha256) · [ملاحظات الإصدار](https://github.com/ilsd7/UnfocusMute/releases/latest) |

بعد فك الضغط، انقل مجلد <code dir="ltr">UnfocusMute-windows-x64</code> إلى المكان الذي تريد حفظ التطبيق فيه، ثم شغّل <code dir="ltr">UnfocusMute-v&lt;version&gt;.exe</code> من داخله.

UnfocusMute تطبيق مستقل لا يحتاج إلى تثبيت. ولا تحتاج أيضًا إلى تثبيت Rust أو Visual Studio Build Tools أو MinGW أو أي أدوات تطوير أخرى.

> **ملاحظة:** بسبب تكلفة شهادات توقيع الكود، يُوزع التطبيق حاليًا من دون توقيع كود لنظام Windows. قد يظهر تحذير Windows SmartScreen أو تحذير "ناشر غير معروف" عند التشغيل الأول. إذا أردت التحقق من سلامة الملف بنفسك، فراجع [الشفافية والتحقق من ملفات الإصدار](#الشفافية-والتحقق-من-ملفات-الإصدار).

<br>

## ملاحظات قبل الاستخدام

يعتمد UnfocusMute على أسماء العمليات ومعلومات النافذة الموجودة في المقدمة وجلسات CoreAudio التي يوفرها Windows. لذلك، إذا قيّدت برامج التشغيل أو إعدادات الأذونات أو برامج الأمان الوصول إلى الجلسات الصوتية، فقد لا يعمل التحكم في الكتم بشكل صحيح.

نوصي بإبقاء UnfocusMute قيد التشغيل في علبة النظام بدلًا من إغلاقه. إذا أُغلق تطبيق مسجل وهو مكتوم، فقد تظل آخر حالة كتم محفوظة. ما دام UnfocusMute قيد التشغيل، فسيستعيد صوت التطبيق تلقائيًا عند تشغيله من جديد ونقله إلى المقدمة. أما إذا أنهيت UnfocusMute أيضًا، فقد لا يصدر التطبيق أي صوت؛ وفي هذه الحالة، ألغِ كتمه يدويًا من <code dir="rtl">خالط مستوى الصوت</code> في Windows.

**سلوك التسجيل باستخدام PID:** لا يقدّم Windows دائمًا القيمة نفسها لمعرّف العملية (PID) الخاص بالجلسة الصوتية ومعرّف العملية الخاص بالنافذة الموجودة في المقدمة. للتعويض عن ذلك، يعتبر UnfocusMute أن التطبيق عاد إلى المقدمة عندما يطابق اسم <code dir="ltr">.exe</code> الخاص بـ PID المسجل اسم <code dir="ltr">.exe</code> للنافذة الحالية في المقدمة.

لذلك، إذا كانت عدة نسخ من ملف <code dir="ltr">.exe</code> نفسه تعمل في الوقت نفسه، فقد لا يتمكن UnfocusMute من تمييز نسخة معيّنة بدقة كاملة. في هذه الحالة قد يعود الصوت عندما تكون نسخة أخرى في المقدمة.

**التوافق مع أنظمة مكافحة الغش:** لا يحقن UnfocusMute أي كود داخل الألعاب، ولا يقرأ ذاكرة اللعبة، ولا يعترض الإدخال، ولا يغيّر ملفات اللعبة. يستخدم فقط معلومات العملية والنافذة الموجودة في المقدمة في Windows ووظائف كتم جلسات CoreAudio، لذلك فهو مصمم لتجنب التعارض مع معظم أنظمة مكافحة الغش، لكن لا يمكن ضمان التوافق معها جميعًا.

<br>

## الاستخدام

1. شغّل UnfocusMute.
2. اختر اللغة. نوصي بالإبقاء على الخيارات بإعداداتها الافتراضية.
3. شغّل اللعبة أو التطبيق الذي تريد تسجيله.
4. اختر تطبيقًا من القائمة أو أدخل اسم <code dir="ltr">.exe</code> الدقيق، ثم اضغط `تسجيل`. إذا لم يكن التطبيق قد أنشأ جلسة صوتية بعد، فانتقل إلى `كل العمليات` لاستعراض جميع العمليات الجارية.
5. إذا احتجت إلى تسجيل PID محدد فقط، اضغط `عرض PID` واختر الإدخال المطلوب. التسجيل باستخدام PID ينطبق فقط على النسخة الحالية قيد التشغيل؛ إذا أُعيد تشغيل التطبيق وتغيّر PID، فستحتاج إلى تسجيله من جديد.
6. انقر بزر الفأرة الأيمن على تطبيق مسجل لتعديل ملاحظته أو استخدام `إيقاف مؤقت` لذلك التطبيق فقط.
7. انقر على حالة `المراقبة نشطة` في الأعلى لإيقاف المراقبة مؤقتًا أو استئنافها.
8. افتح `الإعدادات` لتغيير الخيارات، بما فيها سلوك زر إغلاق النافذة.
9. افتراضيًا، يؤدي إغلاق النافذة إلى إبقاء UnfocusMute قيد التشغيل في علبة النظام. لإنهائه بالكامل، انقر بزر الفأرة الأيمن على أيقونته في علبة النظام واختر `إنهاء`. يمكنك تغيير سلوك زر الإغلاق من `الإعدادات`.

<br>

## استخدام ملاحظات التطبيقات المسجلة

إذا لم يكن اسم العملية وحده كافيًا لتمييز التطبيق، فانقر بزر الفأرة الأيمن على التطبيق المسجل واختر `تعديل الملاحظة`. تظهر الملاحظة فوق اسم العملية في قائمة التطبيقات المسجلة، ولا تؤثر في طريقة مطابقة التطبيق.

هذا مفيد عندما يفتح مشغل لعبة واحد عدة عمليات، أو عندما لا يوضح اسم الملف التنفيذي وظيفته.

- <code dir="ltr">htgame.exe - NTE</code>
- <span dir="ltr"><code>game.exe (PID 21976)</code> - <bdi dir="rtl">عميل خادم الاختبار</bdi></span>

تُحفظ الملاحظات محليًا مع بقية الإعدادات في <code dir="ltr">%APPDATA%\UnfocusMute\config.json</code>.

<br>

## معرفة اسم الملف التنفيذي

إذا لم تكن متأكدًا من الاسم الذي يجب تسجيله، فتحقق من اسم الملف التنفيذي الذي ينتهي بـ <code dir="ltr">.exe</code> في إدارة المهام.

1. شغّل أولًا التطبيق الذي تريد تسجيله.
2. استخدم <span dir="ltr"><kbd>Alt</kbd>+<kbd>Tab</kbd></span> أو <span dir="ltr"><kbd>Windows</kbd>+<kbd>Tab</kbd></span> للعودة إلى سطح مكتب Windows.
3. اضغط <span dir="ltr"><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>Esc</kbd></span> لفتح إدارة المهام.
4. رتّب قائمة العمليات حسب <code dir="ltr">CPU</code> للعثور على التطبيق الذي شغّلته للتو.
5. انقر بزر الفأرة الأيمن على ذلك العنصر وافتح `خصائص`.
6. ابحث عن اسم الملف التنفيذي المنتهي بـ <code dir="ltr">.exe</code>، مثل <code dir="ltr">game.exe</code>، ثم سجّله في UnfocusMute.

<br>

## استكشاف الأخطاء وإصلاحها

إذا لم يظهر تطبيق في القائمة، أو لم يعمل التسجيل باستخدام PID كما تتوقع، فراجع أولًا [ملاحظات قبل الاستخدام](#ملاحظات-قبل-الاستخدام) و[معرفة اسم الملف التنفيذي](#معرفة-اسم-الملف-التنفيذي) أعلاه.

إذا تغيّرت الحالة في الأعلى إلى `تنبيه`، فاضغط `التفاصيل` لعرض رسالة الخطأ التفصيلية.

إذا استمرت المشكلة، فافتح بلاغًا على GitHub.

إذا كنت تشتبه في وجود ثغرة أمنية، فلا تنشر التفاصيل في بلاغ عام. استخدم عملية الإبلاغ الخاصة، وراجع [SECURITY.md](../SECURITY.md) لمزيد من المعلومات.

<br>

## ملف الإعدادات

إذا احتجت إلى فحص ملف الإعدادات مباشرة أو نسخه احتياطيًا، فانقر زر `فتح مجلد الإعدادات` من شاشة الإعدادات. يفتح مستكشف الملفات مجلد <code dir="ltr">%APPDATA%\UnfocusMute</code> حيث تُحفظ الإعدادات.

يمكنك تعديل ملف الإعدادات مباشرة، لكن إذا كان تنسيقه غير صالح ولا يمكن قراءته، فسيُحفظ كنسخة احتياطية باسم <code dir="ltr">config.invalid-&lt;timestamp&gt;.json</code>. إذا اكتُشفت المشكلة أثناء بدء التطبيق، فستُعاد الإعدادات إلى القيم الافتراضية؛ وإذا اكتُشفت أثناء تشغيل التطبيق، فسيُنشأ ملف إعدادات جديد بناءً على إعدادات التطبيق الحالية.

<br>

## الأمان والخصوصية

UnfocusMute تطبيق يعمل محليًا بالكامل. يعمل بشكل طبيعي حتى من دون اتصال بالإنترنت، ولا يحتاج إلى صلاحيات مسؤول. كما أنه لا يرسل طلبات شبكة تلقائية، ولا يستخدم القياس عن بُعد، ولا يرسل تقارير أعطال، ولا يسجل أي شيء عن بُعد، ولا يجمع البيانات.

الاستثناء الوحيد: إذا نقرت بنفسك على زر `مستودع GitHub` في شاشة الإعدادات، يُفتح مستودع GitHub الخاص بهذا المشروع في المتصفح الافتراضي.

اكتشاف الجلسات الصوتية والتحكم في الكتم يستخدمان واجهات Windows CoreAudio فقط. لا يحقن UnfocusMute كودًا داخل العمليات المستهدفة، ولا يقرأ ذاكرتها، ولا يعترض الإدخال.

### المعلومات التي تُحفظ

يحفظ UnfocusMute فقط الإعدادات اللازمة لعمله في <code dir="ltr">%APPDATA%\UnfocusMute\config.json</code>.

- أسماء العمليات المسجلة
- معرّفات PID التي تسجلها مباشرة
- آخر حالة كتم للتطبيقات المسجلة
- الملاحظات التي تكتبها
- اللغة والإعدادات المختارة
- موضع النافذة وحجمها

لا تُرسل هذه المعلومات إلى أي مكان.

لكن إذا فعّلت التشغيل التلقائي عند تسجيل الدخول إلى Windows، فسيُحفظ مسار الملف التنفيذي الحالي أيضًا في قيمة `UnfocusMute` ضمن <code dir="ltr">HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run</code>.

### المعلومات التي لا تُحفظ

لا يحفظ UnfocusMute سجل الاستخدام، أو سجلات النشاط، أو سجلات الأخطاء، أو بيانات الصوت، أو عناوين النوافذ، أو ضغطات المفاتيح، أو أي معلومات غير مذكورة أعلاه ضمن «المعلومات التي تُحفظ».

### طريقة الحذف

لإزالة كل الملفات المتعلقة بالتطبيق، احذف مجلد <code dir="ltr">UnfocusMute-windows-x64</code> ثم احذف <code dir="ltr">%APPDATA%\UnfocusMute</code>.

إذا سبق أن فعّلت التشغيل التلقائي، فاحذف أيضًا قيمة `UnfocusMute` ضمن <code dir="ltr">HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run</code>.

<br>

## الشفافية والتحقق من ملفات الإصدار

صُمم UnfocusMute ليُستخدم بأمان في البيئات المعتادة، لذلك لا يحتاج معظم المستخدمين إلى تنفيذ خطوات التحقق أدناه بشكل منفصل. إذا كنت لا تريد الاعتماد على الثقة بالمطور وحدها أو كنت تولي أمن سلسلة توريد البرمجيات أهمية خاصة، فيمكنك استخدام هذه الخطوات المنشورة للتحقق من مصدر الملفات التي نزّلتها وسلامتها.

### لماذا يلزم تحقق منفصل

حتى لو راجعت الكود المصدري للمستودع بنفسك ووجدته آمنًا، فلا يمكنك الجزم بأن الملفات المنشورة في إصدار GitHub قد بُنيت فعلًا من ذلك المصدر. فإذا تم اختراق حساب المطور أو أسيء استخدام صلاحيات نشر الإصدارات، فقد تُوزع ملفات لا علاقة لها بالكود المصدري المنشور.

يمكن لمقارنة تجزئات SHA-256 تأكيد مطابقة الملف المنزّل لملف التحقق المنشور، لكنها لا تثبت الكود المصدري وبيئة البناء اللذين أُنشئ منهما الملف.

وللتعامل بشفافية مع مخاطر سلسلة التوريد هذه، يوضّح UnfocusMute كيفية التحقق مباشرة من أن ملف إصدار GitHub هو ناتج بناء رسمي أنشأته GitHub Actions من نسخة المصدر التي يشير إليها وسم ذلك الإصدار.

<details>
<summary>عرض خطوات التحقق من ملفات الإصدار</summary>

ثبّت أولًا [GitHub CLI](https://cli.github.com/). ثم نفّذ الأوامر التالية في PowerShell وأدخل إصدار النشر الفعلي عند ظهور المطالبة.

<pre dir="ltr"><code>$version = Read-Host "أدخل إصدار النشر (مثال: v1.5.0)"
$sourceRef = "refs/tags/$version"
$workflow = "ilsd7/UnfocusMute/.github/workflows/release.yml"

gh attestation verify .\UnfocusMute-windows-x64.zip `
  -R ilsd7/UnfocusMute `
  --source-ref $sourceRef `
  --signer-workflow $workflow</code></pre>

يتصل هذا الأمر بخدمة attestations في GitHub ويتحقق من أن تجزئة SHA-256 لملف ZIP المحلي تطابق القيمة المسجلة في إثبات منشأ البناء الموقّع من GitHub Actions.

ويمكنك أيضًا مطابقة تجزئة ملف ZIP مع قيمة SHA-256 المنشورة ضمن الإصدار.

<pre dir="ltr"><code>$expectedHash = ((Get-Content .\UnfocusMute-windows-x64.zip.sha256 -TotalCount 1) -split '\s+')[0]
$actualHash = (Get-FileHash .\UnfocusMute-windows-x64.zip -Algorithm SHA256).Hash

if ($actualHash -ne $expectedHash) {
  throw "فشل التحقق من SHA-256."
}

"تم التحقق من SHA-256: $actualHash"</code></pre>

عند نجاح التحقق، يمكنك التأكد من أن ملف ZIP الذي نزّلته أنشأه سير عمل GitHub Actions المحدد من المصدر المرتبط بوسم الإصدار، وأن تجزئته تطابق القيمة المسجلة في attestation.

لكن هذا لا يثبت سلامة الكود المصدري نفسه، ولا سلامة بيئة GitHub بأكملها، ولا أن البناء قابل لإعادة الإنتاج بايتًا ببايت على جهاز آخر.

</details>

<br>

## البناء من المصدر

هدف الإصدار الموصى به هو <code dir="ltr">x86_64-pc-windows-msvc</code>.

المتطلبات:

- Rust stable
- Visual Studio Build Tools 2022 أو Visual Studio 2022
- Windows 10/11 SDK

<details>
<summary>عرض أوامر البناء وإنشاء الحزمة</summary>

<pre dir="ltr"><code>rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc --locked</code></pre>

إعدادات بناء الإصدار مضبوطة لتقليل حجم الملف الناتج. يزيل ملف تعريف الإصدار في <code dir="ltr">Cargo.toml</code> الرموز، ويفعّل LTO، ويستخدم وحدة توليد كود واحدة (codegen unit)، ويضبط <code dir="ltr">panic = "abort"</code>، ويستخدم تحسينات موجهة لتقليل الحجم.

الملف التنفيذي:

<pre dir="ltr"><code>target\x86_64-pc-windows-msvc\release\unfocusmute.exe</code></pre>

ملف ZIP للتوزيع:

<pre dir="ltr"><code>powershell -NoProfile -ExecutionPolicy Bypass -File scripts\package-windows.ps1</code></pre>

تحديث إشعارات تراخيص الطرف الثالث:

<pre dir="ltr"><code>cargo about generate about.hbs -c about.toml --locked --offline -o THIRD_PARTY_NOTICES.md</code></pre>

يتم إنشاء الناتج في <code dir="ltr">dist\UnfocusMute-windows-x64.zip</code>، ومعه ملف تحقق SHA-256 في <code dir="ltr">dist\UnfocusMute-windows-x64.zip.sha256</code>. يحتوي ZIP على الملف التنفيذي الذي يتضمن رقم الإصدار (<code dir="ltr">UnfocusMute-v&lt;version&gt;.exe</code>)، و<code dir="ltr">LICENSE</code>، و<code dir="ltr">THIRD_PARTY_NOTICES.md</code>، وملفات README بصيغة <code dir="ltr">.txt</code> لكل لغة داخل مجلد <code dir="ltr">docs</code>.

</details>

<br>

## الترخيص

Apache License 2.0. راجع [LICENSE](../LICENSE) للتفاصيل.

راجع [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md) لإشعارات تراخيص حزم Rust الخارجية.

</div>
