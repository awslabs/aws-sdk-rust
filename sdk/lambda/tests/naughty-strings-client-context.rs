#![allow(dead_code)]
/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

// use http_1x::HeaderValue;

const NAUGHTY_STRINGS: &str = include_str!("blns/blns.txt");

/// A list of lines to skipped when iterating over the blns. These lines should all fail when
/// entered into the AWS CLI too. In the below test, every one of these lines will produce an
/// `InvalidSignatureException` error with the message:
/// > The request signature we calculated does not match the signature you provided. Check your AWS
/// > Secret Access Key and signing method. Consult the service documentation for details.
const SKIPPED_LINES: &[usize] = &[
    124, // ''
    139, // `cargo build` can't handle this one
    143, // '﻿'
    144, // '￾'
    150, // 'Ω≈ç√∫˜µ≤≥÷'
    151, // 'åß∂ƒ©˙∆˚¬…æ'
    152, // 'œ∑´®†¥¨ˆøπ“‘'
    153, // '¡™£¢∞§¶•ªº–≠'
    154, // '¸˛Ç◊ı˜Â¯˘¿'
    155, // 'ÅÍÎÏ˝ÓÔÒÚÆ☃'
    156, // 'Œ„´‰ˇÁ¨ˆØ∏”’'
    157, // '`⁄€‹›ﬁﬂ‡°·‚—±'
    158, // '⅛⅜⅝⅞'
    159, // 'ЁЂЃЄЅІЇЈЉЊЋЌЍЎЏАБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдежзийклмнопрстуфхцчшщъыьэюя'
    160, // '٠١٢٣٤٥٦٧٨٩'
    166, // '⁰⁴⁵'
    167, // '₀₁₂'
    168, // '⁰⁴⁵₀₁₂'
    169, // 'ด้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็ ด้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็ ด้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็็้้้้้้้้็็็็็้้้้้็็็็'
    182, // '<foo val=“bar” />'
    183, // '<foo val=“bar” />'
    184, // '<foo val=”bar“ />'
    191, // '田中さんにあげて下さい'
    192, // 'パーティーへ行かないか'
    193, // '和製漢語'
    194, // '部落格'
    195, // '사회과학원 어학연구소'
    196, // '찦차를 타고 온 펲시맨과 쑛다리 똠방각하'
    197, // '社會科學院語學研究所'
    198, // '울란바토르'
    199, // '𠜎𠜱𠝹𠱓𠱸𠲖𠳏'
    203, // '𐐜 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐙𐐊𐐡𐐝𐐓/𐐝𐐇𐐗𐐊𐐤𐐔 𐐒𐐋𐐗 𐐒𐐌 𐐜 𐐡𐐀𐐖𐐇𐐤𐐓𐐝 𐐱𐑂 𐑄 𐐔𐐇𐐝𐐀𐐡𐐇𐐓 𐐏𐐆𐐅𐐤𐐆𐐚𐐊𐐡𐐝𐐆𐐓𐐆'
    227, // '表ポあA鷗ŒéＢ逍Üßªąñ丂㐀𠀀'
    234, // 'Ⱥ'
    235, // 'Ⱦ'
    241, // 'ヽ༼ຈل͜ຈ༽ﾉ ヽ༼ຈل͜ຈ༽ﾉ'
    242, // '(｡◕ ∀ ◕｡)'
    243, // '｀ｨ(´∀｀∩'
    244, // '__ﾛ(,_,*)'
    245, // '・(￣∀￣)・:*:'
    246, // 'ﾟ･✿ヾ╲(｡◕‿◕｡)╱✿･ﾟ'
    247, // ',。・:*:・゜’( ☻ ω ☻ )。・:*:・゜’'
    248, // '(╯°□°）╯︵ ┻━┻)'
    249, // '(ﾉಥ益ಥ）ﾉ﻿ ┻━┻'
    250, // '┬─┬ノ( º _ ºノ)'
    251, // '( ͡° ͜ʖ ͡°)'
    252, // '¯\_(ツ)_/¯'
    258, // '😍'
    259, // '👩🏽'
    260, // '👨‍🦰 👨🏿‍🦰 👨‍🦱 👨🏿‍🦱 🦹🏿‍♂️'
    261, // '👾 🙇 💁 🙅 🙆 🙋 🙎 🙍'
    262, // '🐵 🙈 🙉 🙊'
    263, // '❤️ 💔 💌 💕 💞 💓 💗 💖 💘 💝 💟 💜 💛 💚 💙'
    264, // '✋🏿 💪🏿 👐🏿 🙌🏿 👏🏿 🙏🏿'
    265, // '👨‍👩‍👦 👨‍👩‍👧‍👦 👨‍👨‍👦 👩‍👩‍👧 👨‍👦 👨‍👧‍👦 👩‍👦 👩‍👧‍👦'
    266, // '🚾 🆒 🆓 🆕 🆖 🆗 🆙 🏧'
    267, // '0️⃣ 1️⃣ 2️⃣ 3️⃣ 4️⃣ 5️⃣ 6️⃣ 7️⃣ 8️⃣ 9️⃣ 🔟'
    274, // '🇺🇸🇷🇺🇸 🇦🇫🇦🇲🇸'
    275, // '🇺🇸🇷🇺🇸🇦🇫🇦🇲'
    276, // '🇺🇸🇷🇺🇸🇦'
    282, // '１２３'
    283, // '١٢٣'
    289, // 'ثم نفس سقطت وبالتحديد،, جزيرتي باستخدام أن دنو. إذ هنا؟ الستار وتنصيب كان. أهّل ايطاليا، بريطانيا-فرنسا قد أخذ. سليمان، إتفاقية بين ما, يذكر الحدود أي بعد, معاملة بولندا، الإطلاق عل إيو.'
    290, // 'בְּרֵאשִׁית, בָּרָא אֱלֹהִים, אֵת הַשָּׁמַיִם, וְאֵת הָאָרֶץ'
    291, // 'הָיְתָהtestالصفحات التّحول'
    292, // '﷽'
    293, // 'ﷺ'
    294, // 'مُنَاقَشَةُ سُبُلِ اِسْتِخْدَامِ اللُّغَةِ فِي النُّظُمِ الْقَائِمَةِ وَفِيم يَخُصَّ التَّطْبِيقَاتُ الْحاسُوبِيَّةُ،'
    295, // 'الكل في المجمو عة (5)'
    301, // '᚛ᚄᚓᚐᚋᚒᚄ ᚑᚄᚂᚑᚏᚅ᚜'
    302, // '᚛                 ᚜'
    308, // `cargo build` can't handle this one
    309, // `cargo build` can't handle this one
    310, // I couldn't paste this one because my IDE parsed it as a syntax error
    311, // `cargo build` can't handle this one
    312, // `cargo build` can't handle this one
    318, // 'Ṱ̺̺̕o͞ ̷i̲̬͇̪͙n̝̗͕v̟̜̘̦͟o̶̙̰̠kè͚̮̺̪̹̱̤ ̖t̝͕̳̣̻̪͞h̼͓̲̦̳̘̲e͇̣̰̦̬͎ ̢̼̻̱̘h͚͎͙̜̣̲ͅi̦̲̣̰̤v̻͍e̺̭̳̪̰-m̢iͅn̖̺̞̲̯̰d̵̼̟͙̩̼̘̳ ̞̥̱̳̭r̛̗̘e͙p͠r̼̞̻̭̗e̺̠̣͟s̘͇̳͍̝͉e͉̥̯̞̲͚̬͜ǹ̬͎͎̟̖͇̤t͍̬̤͓̼̭͘ͅi̪̱n͠g̴͉ ͏͉ͅc̬̟h͡a̫̻̯͘o̫̟̖͍̙̝͉s̗̦̲.̨̹͈̣'
    319, // '̡͓̞ͅI̗̘̦͝n͇͇͙v̮̫ok̲̫̙͈i̖͙̭̹̠̞n̡̻̮̣̺g̲͈͙̭͙̬͎ ̰t͔̦h̞̲e̢̤ ͍̬̲͖f̴̘͕̣è͖ẹ̥̩l͖͔͚i͓͚̦͠n͖͍̗͓̳̮g͍ ̨o͚̪͡f̘̣̬ ̖̘͖̟͙̮c҉͔̫͖͓͇͖ͅh̵̤̣͚͔á̗̼͕ͅo̼̣̥s̱͈̺̖̦̻͢.̛̖̞̠̫̰'
    320, // '̗̺͖̹̯͓Ṯ̤͍̥͇͈h̲́e͏͓̼̗̙̼̣͔ ͇̜̱̠͓͍ͅN͕͠e̗̱z̘̝̜̺͙p̤̺̹͍̯͚e̠̻̠͜r̨̤͍̺̖͔̖̖d̠̟̭̬̝͟i̦͖̩͓͔̤a̠̗̬͉̙n͚͜ ̻̞̰͚ͅh̵͉i̳̞v̢͇ḙ͎͟-҉̭̩̼͔m̤̭̫i͕͇̝̦n̗͙ḍ̟ ̯̲͕͞ǫ̟̯̰̲͙̻̝f ̪̰̰̗̖̭̘͘c̦͍̲̞͍̩̙ḥ͚a̮͎̟̙͜ơ̩̹͎s̤.̝̝ ҉Z̡̖̜͖̰̣͉̜a͖̰͙̬͡l̲̫̳͍̩g̡̟̼̱͚̞̬ͅo̗͜.̟'
    321, // '̦H̬̤̗̤͝e͜ ̜̥̝̻͍̟́w̕h̖̯͓o̝͙̖͎̱̮ ҉̺̙̞̟͈W̷̼̭a̺̪͍į͈͕̭͙̯̜t̶̼̮s̘͙͖̕ ̠̫̠B̻͍͙͉̳ͅe̵h̵̬͇̫͙i̹͓̳̳̮͎̫̕n͟d̴̪̜̖ ̰͉̩͇͙̲͞ͅT͖̼͓̪͢h͏͓̮̻e̬̝̟ͅ ̤̹̝W͙̞̝͔͇͝ͅa͏͓͔̹̼̣l̴͔̰̤̟͔ḽ̫.͕'
    322, // 'Z̮̞̠͙͔ͅḀ̗̞͈̻̗Ḷ͙͎̯̹̞͓G̻O̭̗̮'
    328, // '˙ɐnbᴉlɐ ɐuƃɐɯ ǝɹolop ʇǝ ǝɹoqɐl ʇn ʇunpᴉpᴉɔuᴉ ɹodɯǝʇ poɯsnᴉǝ op pǝs 'ʇᴉlǝ ƃuᴉɔsᴉdᴉpɐ ɹnʇǝʇɔǝsuoɔ 'ʇǝɯɐ ʇᴉs ɹolop ɯnsdᴉ ɯǝɹo˥'
    329, // '00˙Ɩ$-'
    335, // 'Ｔｈｅ ｑｕｉｃｋ ｂｒｏｗｎ ｆｏｘ ｊｕｍｐｓ ｏｖｅｒ ｔｈｅ ｌａｚｙ ｄｏｇ'
    336, // '𝐓𝐡𝐞 𝐪𝐮𝐢𝐜𝐤 𝐛𝐫𝐨𝐰𝐧 𝐟𝐨𝐱 𝐣𝐮𝐦𝐩𝐬 𝐨𝐯𝐞𝐫 𝐭𝐡𝐞 𝐥𝐚𝐳𝐲 𝐝𝐨𝐠'
    337, // '𝕿𝖍𝖊 𝖖𝖚𝖎𝖈𝖐 𝖇𝖗𝖔𝖜𝖓 𝖋𝖔𝖝 𝖏𝖚𝖒𝖕𝖘 𝖔𝖛𝖊𝖗 𝖙𝖍𝖊 𝖑𝖆𝖟𝖞 𝖉𝖔𝖌'
    338, // '𝑻𝒉𝒆 𝒒𝒖𝒊𝒄𝒌 𝒃𝒓𝒐𝒘𝒏 𝒇𝒐𝒙 𝒋𝒖𝒎𝒑𝒔 𝒐𝒗𝒆𝒓 𝒕𝒉𝒆 𝒍𝒂𝒛𝒚 𝒅𝒐𝒈'
    339, // '𝓣𝓱𝓮 𝓺𝓾𝓲𝓬𝓴 𝓫𝓻𝓸𝔀𝓷 𝓯𝓸𝔁 𝓳𝓾𝓶𝓹𝓼 𝓸𝓿𝓮𝓻 𝓽𝓱𝓮 𝓵𝓪𝔃𝔂 𝓭𝓸𝓰'
    340, // '𝕋𝕙𝕖 𝕢𝕦𝕚𝕔𝕜 𝕓𝕣𝕠𝕨𝕟 𝕗𝕠𝕩 𝕛𝕦𝕞𝕡𝕤 𝕠𝕧𝕖𝕣 𝕥𝕙𝕖 𝕝𝕒𝕫𝕪 𝕕𝕠𝕘'
    341, // '𝚃𝚑𝚎 𝚚𝚞𝚒𝚌𝚔 𝚋𝚛𝚘𝚠𝚗 𝚏𝚘𝚡 𝚓𝚞𝚖𝚙𝚜 𝚘𝚟𝚎𝚛 𝚝𝚑𝚎 𝚕𝚊𝚣𝚢 𝚍𝚘𝚐'
    342, // '⒯⒣⒠ ⒬⒰⒤⒞⒦ ⒝⒭⒪⒲⒩ ⒡⒪⒳ ⒥⒰⒨⒫⒮ ⒪⒱⒠⒭ ⒯⒣⒠ ⒧⒜⒵⒴ ⒟⒪⒢'
    357, // ' onfocus=JaVaSCript:alert(9) autofocus'
    360, // '＜script＞alert(12)＜/script＞'
    564, // '<IMG SRC="jav   ascript:alert('214');">'
    569, // '<IMG SRC=" &#14;  javascript:alert('219');">'
    729, // 'Powerلُلُصّبُلُلصّبُررً ॣ ॣh ॣ ॣ冗'
    730, // '🏳0🌈️'
    731, // 'జ్ఞ‌ా'
    737, // 'گچپژ'
];

// #[tokio::test]
// async fn test_client_context_field_against_naughty_strings_list() {
//     tracing_subscriber::fmt::init();
//
//     // re-add `aws-config = { path = "../../build/aws-sdk/aws-config" }` to this project's Cargo.toml
//     let config = aws_config::load_from_env().await;
//     let client = aws_sdk_lambda::Client::new(&config);
//     let invalid_request_content_exception = "InvalidRequestContentException: Client context must be a valid Base64-encoded JSON object.";
//     let unrecognized_client_exception =
//         "UnrecognizedClientException: The security token included in the request is invalid.";
//
//     let mut encountered_errors = false;
//
//     for (idx, line) in NAUGHTY_STRINGS.split('\n').enumerate() {
//         // Some lines in blns aren't even accepted by the AWS CLI so it's reasonable to skip them
//         if SKIPPED_LINES.contains(&(idx + 1)) {
//             continue;
//         }
//
//         // add lines to metadata unless they're a comment or empty
//         // Some naughty strings aren't valid HeaderValues so we skip those too
//         if !line.starts_with("#") && !line.is_empty() && HeaderValue::from_str(line).is_ok() {
//             let err = client
//                 .invoke()
//                 .function_name("testFunctionThatDoesNothing")
//                 .client_context(line)
//                 .send()
//                 .await
//                 .unwrap_err();
//
//             match err.to_string() {
//                 // If this happens, it means that someone tried to run the test without valid creds
//                 err if err == unrecognized_client_exception => {
//                     panic!("Set valid credentials before running this test.");
//                 }
//                 // This is the expected error so we ignore it and continue
//                 err if err == invalid_request_content_exception => continue,
//                 // Other errors are bad and so we bring attention to them
//                 err => {
//                     encountered_errors = true;
//                     // 1 is added to idx because line numbers start at one
//                     eprintln!(
//                         "line {} '{}' caused unexpected error: {}",
//                         idx + 1,
//                         line,
//                         err
//                     );
//                 }
//             }
//         }
//     }
//
//     if encountered_errors {
//         panic!(
//             "one or more errors were encountered while testing lambda invoke with naughty strings"
//         );
//     }
// }
