//! Curated collections of test data for property-based testing
//!
//! This module provides curated collections of strings and data that are useful
//! for property-based testing. These collections are ported from the Haskell
//! Hedgehog corpus and provide realistic test data for various domains.
//! 
//! # Usage
//! 
//! ```rust
//! use hedgehog::*;
//! use hedgehog::corpus;
//! 
//! // Use generator functions for convenient access
//! let muppet_gen = corpus::gen::muppet();
//! let animal_gen = corpus::gen::animal();
//! 
//! // Test unicode handling with the glass collection
//! let unicode_gen = corpus::gen::glass();
//! let prop = for_all(unicode_gen, |text: &&str| {
//!     // Test that your unicode handling works correctly
//!     text.chars().count() > 0
//! });
//! 
//! // Or use the collections directly if needed
//! let custom_gen = Gen::new(|_size, seed| {
//!     let idx = seed.next_bounded(corpus::MUPPETS.len() as u64).0 as usize;
//!     Tree::singleton(corpus::MUPPETS[idx])
//! });
//! ```

use crate::*;

/// Collection of Muppets characters
pub const MUPPETS: &[&str] = &[
    "kermit",
    "gonzo", 
    "fozzy",
    "chef",
    "statler",
    "waldorf",
    "beaker",
    "animal",
];

/// Collection of cooking-related terms
pub const COOKING: &[&str] = &[
    "salted",
    "stewed",
    "diced",
    "filleted",
    "sauteed",
];

/// Collection of animals
pub const ANIMALS: &[&str] = &[
    "alligator", "ant", "bear", "bee", "bird", "camel", "cat", "cheetah",
    "chicken", "chimpanzee", "cow", "crocodile", "deer", "dog", "dolphin",
    "duck", "eagle", "elephant", "fish", "fly", "fox", "frog", "giraffe",
    "goat", "goldfish", "hamster", "hippopotamus", "horse", "kangaroo",
    "kitten", "lion", "lobster", "monkey", "octopus", "owl", "panda",
    "pig", "puppy", "rabbit", "rat", "scorpion", "seal", "shark", "sheep",
    "snail", "snake", "spider", "squirrel", "tiger", "turtle", "wolf", "zebra",
];

/// Collection of colors
pub const COLOURS: &[&str] = &[
    "red", "green", "blue", "yellow", "black", "grey", "purple", "orange", "pink",
];

/// Collection of fruits
pub const FRUITS: &[&str] = &[
    "apple", "banana", "cherry", "grapefruit", "grapes", "lemon", "lime",
    "melon", "orange", "peach", "pear", "persimmon", "pineapple", "plum",
    "strawberry", "tangerine", "tomato", "watermelon",
];

/// Collection of vegetables
pub const VEGETABLES: &[&str] = &[
    "asparagus", "beans", "broccoli", "cabbage", "carrot", "celery", "corn",
    "cucumber", "eggplant", "green pepper", "lettuce", "onion", "peas",
    "potato", "pumpkin", "radish", "spinach", "sweet potato", "tomato", "turnip",
];

/// Collection of weather conditions
pub const WEATHER: &[&str] = &[
    "dry", "raining", "hot", "humid", "snowing", "fresh", "windy", "freezing",
];

/// Collection of bodies of water
pub const WATERS: &[&str] = &[
    "basin", "bay", "billabong", "canal", "channel", "creek", "estuary",
    "fjord", "harbour", "lake", "loch", "marsh", "ocean", "pond", "puddle",
    "reservoir", "river", "sea", "slough", "sound", "spring", "stream",
    "swamp", "wetland",
];

/// Collection of metasyntactic variables
pub const METASYNTACTIC: &[&str] = &[
    "foo", "bar", "baz", "qux", "quux", "quuz", "corge", "grault",
    "garply", "waldo", "fred", "plugh", "xyzzy", "thud",
];

/// The famous "I can eat glass" phrase in many different languages and scripts.
/// This is an excellent test corpus for unicode handling, covering a wide range
/// of scripts, writing systems, and character encodings.
///
/// From: http://kermitproject.org/utf8.html
pub const GLASS: &[&str] = &[
    // Sanskrit
    "काचं शक्नोम्यत्तुम् । नोपहिनस्ति माम् ॥",
    "kācaṃ śaknomyattum; nopahinasti mām.",
    
    // Greek
    "ὕαλον ϕαγεῖν δύναμαι· τοῦτο οὔ με βλάπτει.",
    "Μπορώ να φάω σπασμένα γυαλιά χωρίς να πάθω τίποτα.",
    "Μπορῶ νὰ φάω σπασμένα γυαλιὰ χωρὶς νὰ πάθω τίποτα. ",
    
    // Latin and Romance languages
    "Vitrum edere possum; mihi non nocet.",
    "Je puis mangier del voirre. Ne me nuit.",
    "Je peux manger du verre, ça ne me fait pas mal.",
    "Pòdi manjar de veire, me nafrariá pas.",
    "J'peux manger d'la vitre, ça m'fa pas mal.",
    "Dji pou magnî do vêre, çoula m' freut nén må. ",
    "Ch'peux mingi du verre, cha m'foé mie n'ma. ",
    "Mwen kap manje vè, li pa blese'm.",
    
    // Iberian languages
    "Kristala jan dezaket, ez dit minik ematen.",
    "Puc menjar vidre, que no em fa mal.",
    "Puedo comer vidrio, no me hace daño.",
    "Puedo minchar beire, no me'n fa mal . ",
    "Eu podo xantar cristais e non cortarme.",
    "Posso comer vidro, não me faz mal.",
    "Posso comer vidro, não me machuca.",
    "M' podê cumê vidru, ca ta maguâ-m '.",
    "Ami por kome glas anto e no ta hasimi daño.",
    
    // Italian and variants
    "Posso mangiare il vetro e non mi fa male.",
    "Sôn bôn de magnà el véder, el me fa minga mal.",
    "Me posso magna' er vetro, e nun me fa male.",
    "M' pozz magna' o'vetr, e nun m' fa mal.",
    "Mi posso magnare el vetro, no'l me fa mae.",
    "Pòsso mangiâ o veddro e o no me fà mâ.",
    "Puotsu mangiari u vitru, nun mi fa mali. ",
    "Jau sai mangiar vaider, senza che quai fa donn a mai. ",
    
    // Romanian and Esperanto
    "Pot să mănânc sticlă și ea nu mă rănește.",
    "Mi povas manĝi vitron, ĝi ne damaĝas min. ",
    
    // Celtic languages
    "Mý a yl dybry gwéder hag éf ny wra ow ankenya.",
    "Dw i'n gallu bwyta gwydr, 'dyw e ddim yn gwneud dolur i mi.",
    "Foddym gee glonney agh cha jean eh gortaghey mee.",
    "᚛᚛ᚉᚑᚅᚔᚉᚉᚔᚋ ᚔᚈᚔ ᚍᚂᚐᚅᚑ ᚅᚔᚋᚌᚓᚅᚐ᚜",
    "Con·iccim ithi nglano. Ním·géna.",
    "Is féidir liom gloinne a ithe. Ní dhéanann sí dochar ar bith dom.",
    "Ithim-s a gloine agus ní miste damh é.",
    "S urrainn dhomh gloinne ithe; cha ghoirtich i mi.",
    
    // Germanic languages (historical and modern)
    "ᛁᚳ᛫ᛗᚨᚷ᛫ᚷᛚᚨᛋ᛫ᛖᚩᛏᚪᚾ᛫ᚩᚾᛞ᛫ᚻᛁᛏ᛫ᚾᛖ᛫ᚻᛖᚪᚱᛗᛁᚪᚧ᛫ᛗᛖ᛬",
    "Ic mæg glæs eotan ond hit ne hearmiað me.",
    "Ich canne glas eten and hit hirtiþ me nouȝt.",
    "I can eat glass and it doesn't hurt me.",
    "[aɪ kæn iːt glɑːs ænd ɪt dɐz nɒt hɜːt miː]",
    "⠊⠀⠉⠁⠝⠀⠑⠁⠞⠀⠛⠇⠁⠎⠎⠀⠁⠝⠙⠀⠊⠞⠀⠙⠕⠑⠎⠝⠞⠀⠓⠥⠗⠞⠀⠍⠑",
    "Mi kian niam glas han i neba hot mi.",
    "Ah can eat gless, it disnae hurt us. ",
    "𐌼𐌰𐌲 𐌲𐌻𐌴𐍃 𐌹̈𐍄𐌰𐌽, 𐌽𐌹 𐌼𐌹𐍃 𐍅𐌿 𐌽𐌳𐌰𐌽 𐌱𐍂𐌹𐌲𐌲𐌹𐌸.",
    
    // Norse and Scandinavian
    "ᛖᚴ ᚷᛖᛏ ᛖᛏᛁ ᚧ ᚷᛚᛖᚱ ᛘᚾ ᚦᛖᛋᛋ ᚨᚧ ᚡᛖ ᚱᚧᚨ ᛋᚨᚱ",
    "Ek get etið gler án þess að verða sár.",
    "Eg kan eta glas utan å skada meg.",
    "Jeg kan spise glass uten å skade meg.",
    "Eg kann eta glas, skaðaleysur.",
    "Ég get etið gler án þess að meiða mig.",
    "Jag kan äta glas utan att skada mig.",
    "Jeg kan spise glas, det gør ikke ondt på mig.",
    "Æ ka æe glass uhen at det go mæ naue.",
    
    // Dutch and related
    "Ik kin glês ite, it docht me net sear.",
    "Ik kan glas eten, het doet mĳ geen kwaad.",
    "Iech ken glaas èèse, mer 't deet miech jing pieng.",
    "Ek kan glas eet, maar dit doen my nie skade nie.",
    "Ech kan Glas iessen, daat deet mir nët wei.",
    
    // German and variants
    "Ich kann Glas essen, ohne mir zu schaden.",
    "Ich kann Glas verkasematuckeln, ohne dattet mich wat jucken tut.",
    "Isch kann Jlaas kimmeln, uuhne datt mich datt weh dääd.",
    "Ich koann Gloos assn und doas dudd merr ni wii.",
    "Iech konn glaasch voschbachteln ohne dass es mir ebbs daun doun dud.",
    "'sch kann Glos essn, ohne dass'sch mer wehtue.",
    "Isch konn Glass fresse ohne dasses mer ebbes ausmache dud.",
    "I kå Glas frässa, ond des macht mr nix!",
    "I ka glas eassa, ohne dass mar weh tuat.",
    "I koh Glos esa, und es duard ma ned wei.",
    "I kaun Gloos essen, es tuat ma ned weh.",
    "Ich chan Glaas ässe, das schadt mir nöd.",
    "Ech cha Glâs ässe, das schadt mer ned. ",
    
    // Finno-Ugric languages
    "Meg tudom enni az üveget, nem lesz tőle bajom.",
    "Voin syödä lasia, se ei vahingoita minua.",
    "Sáhtán borrat lása, dat ii leat bávččas.",
    "Мон ярсан суликадо, ды зыян эйстэнзэ а ули.",
    "Mie voin syvvä lasie ta minla ei ole kipie.",
    "Minä voin syvvä st'oklua dai minule ei ole kibie. ",
    
    // Baltic languages
    "Ma võin klaasi süüa, see ei tee mulle midagi.",
    "Es varu ēst stiklu, tas man nekaitē.",
    "Aš galiu valgyti stiklą ir jis manęs nežeidžia ",
    
    // Slavic languages
    "Mohu jíst sklo, neublíží mi.",
    "Môžem jesť sklo. Nezraní ma.",
    "Mogę jeść szkło i mi nie szkodzi.",
    "Lahko jem steklo, ne da bi mi škodovalo.",
    "Ja mogu jesti staklo, i to mi ne šteti.",
    "Ја могу јести стакло, и то ми не штети.",
    "Можам да јадам стакло, а не ме штета.",
    "Я могу есть стекло, оно мне не вредит.",
    "Я магу есці шкло, яно мне не шкодзіць.",
    "Ja mahu jeści škło, jano mne ne škodzić.",
    "Я можу їсти скло, і воно мені не зашкодить.",
    "Мога да ям стъкло, то не ми вреди.",
    
    // Caucasian and other European
    "მინას ვჭამ და არა მტკივა.",
    "Կրնամ ապակի ուտել և ինծի անհանգիստ չըներ։",
    "Unë mund të ha qelq dhe nuk më gjen gjë.",
    
    // Turkic languages
    "Cam yiyebilirim, bana zararı dokunmaz.",
    "جام ييه بلورم بڭا ضررى طوقونمز",
    "Men shisha yeyishim mumkin, ammo u menga zarar keltirmaydi.",
    "Мен шиша ейишим мумкин, аммо у менга зарар келтирмайди.",
    
    // South Asian languages
    "আমি কাঁচ খেতে পারি, তাতে আমার কোনো ক্ষতি হয় না।",
    "मी काच खाऊ शकतो, मला ते दुखत नाही.",
    "ನನಗೆ ಹಾನಿ ಆಗದೆ, ನಾನು ಗಜನ್ನು ತಿನಬಹುದು",
    "मैं काँच खा सकता हूँ और मुझे उससे कोई चोट नहीं पहुंचती.",
    "എനിക്ക് ഗ്ലാസ് തിന്നാം. അതെന്നെ വേദനിപ്പിക്കില്ല.",
    "நான் கண்ணாடி சாப்பிடுவேன், அதனால் எனக்கு ஒரு கேடும் வராது.",
    "నేను గాజు తినగలను మరియు అలా చేసినా నాకు ఏమి ఇబ్బంది లేదు",
    "මට වීදුරු කෑමට හැකියි. එයින් මට කිසි හානියක් සිදු නොවේ.",
    "میں کانچ کھا سکتا ہوں اور مجھے تکلیف نہیں ہوتی ۔",
    "زه شيشه خوړلې شم، هغه ما نه خوږوي",
    ".من می توانم بدونِ احساس درد شيشه بخورم",
    
    // Middle Eastern languages
    "أنا قادر على أكل الزجاج و هذا لا يؤلمني. ",
    "Nista' niekol il-ħ ġieġ u ma jagħmilli xejn.",
    "אני יכול לאכול זכוכית וזה לא מזיק לי.",
    "איך קען עסן גלאָז און עס טוט מיר נישט װײ. ",
    
    // African languages
    "Metumi awe tumpan, ɜnyɜ me hwee.",
    "Inā iya taunar gilāshi kuma in gamā lāfiyā.",
    "إِنا إِىَ تَونَر غِلَاشِ كُمَ إِن غَمَا لَافِىَا",
    "Mo lè je̩ dígí, kò ní pa mí lára.",
    "Nakokí kolíya biténi bya milungi, ekosála ngáí mabé tɛ́.",
    "Naweza kula bilauri na sikunyui.",
    
    // Southeast Asian languages
    "Saya boleh makan kaca dan ia tidak mencederakan saya.",
    "Kaya kong kumain nang bubog at hindi ako masaktan.",
    "Siña yo' chumocho krestat, ti ha na'lalamen yo'.",
    "Au rawa ni kana iloilo, ia au sega ni vakacacani kina.",
    "Aku isa mangan beling tanpa lara.",
    "က္ယ္ဝန္တော္၊က္ယ္ဝန္မ မ္ယက္စားနုိင္သည္။ ၎က္ရောင္ ထိခုိက္မ္ဟု မရ္ဟိပာ။",
    "ကျွန်တော် ကျွန်မ မှန်စားနိုင်တယ်။ ၎င်းကြောင့် ထိခိုက်မှုမရှိပါ။",
    "Tôi có thể ăn thủy tinh mà không hại gì.",
    "些 𣎏 世 咹 水 晶 𦓡 空 𣎏 害 咦",
    "ខ្ញុំអាចញុំកញ្ចក់បាន ដោយគ្មានបញ្ហារ",
    "ຂອ້ຍກິນແກ້ວໄດ້ໂດຍທີ່ມັນບໍ່ໄດ້ເຮັດໃຫ້ຂອ້ຍເຈັບ.",
    "ฉันกินกระจกได้ แต่มันไม่ทำให้ฉันเจ็บ",
    
    // East Asian languages
    "Би шил идэй чадна, надад хортой биш",
    "ᠪᠢ ᠰᠢᠯᠢ ᠢᠳᠡᠶᠦ ᠴᠢᠳᠠᠨᠠ ᠂ ᠨᠠᠳᠤᠷ ᠬᠣᠤᠷᠠᠳᠠᠢ ᠪᠢᠰᠢ ",
    "म काँच खान सक्छू र मलाई केहि नी हुन्न् ।",
    "ཤེལ་སྒོ་ཟ་ནས་ང་ན་གི་མ་རེད།",
    "我能吞下玻璃而不伤身体。",
    "我能吞下玻璃而不傷身體。",
    "Góa ē-t àng chia̍h po-lê, mā bē tio̍h-siong.",
    "私はガラスを食べられます。それは私を傷つけません。",
    "나는 유리를 먹을 수 있어요. 그래도 아프지 않아요",
    
    // Pacific and constructed languages
    "Mi save kakae glas, hemi no save katem mi.",
    "Hiki iaʻu ke ʻai i ke aniani; ʻaʻole nō lā au e ʻeha.",
    "E koʻana e kai i te karahi, mea ʻā, ʻaʻe hauhau.",
    "ᐊᓕᒍᖅ ᓂᕆᔭᕌᖓᒃᑯ ᓱᕋᙱᑦᑐᓐᓇᖅᑐᖓ",
    "Naika məkmək kakshət labutay, pi weyk ukuk munk-s ik nay.",
    "Tsésǫʼ yishą́ągo bííníshghah dóó doo shił neezgai da. ",
    "mi kakne le nu citka le blaci .iku'i le se go'i na xrani mi",
    "Ljœr ye caudran créneþ ý jor cẃran.",
];

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_corpus_collections_not_empty() {
        assert!(!MUPPETS.is_empty());
        assert!(!COOKING.is_empty());
        assert!(!ANIMALS.is_empty());
        assert!(!COLOURS.is_empty());
        assert!(!FRUITS.is_empty());
        assert!(!VEGETABLES.is_empty());
        assert!(!WEATHER.is_empty());
        assert!(!WATERS.is_empty());
        assert!(!METASYNTACTIC.is_empty());
        assert!(!GLASS.is_empty());
    }
    
    #[test]
    fn test_glass_contains_unicode() {
        // Should contain various scripts and unicode characters
        let has_chinese = GLASS.iter().any(|s| s.contains("玻璃"));
        let has_arabic = GLASS.iter().any(|s| s.contains("الزجاج"));
        let has_russian = GLASS.iter().any(|s| s.contains("стекло"));
        let has_japanese = GLASS.iter().any(|s| s.contains("ガラス"));
        
        assert!(has_chinese, "Should contain Chinese text");
        assert!(has_arabic, "Should contain Arabic text");
        assert!(has_russian, "Should contain Russian text");
        assert!(has_japanese, "Should contain Japanese text");
    }
    
    #[test]
    fn test_muppets_collection() {
        assert!(MUPPETS.contains(&"kermit"));
        assert!(MUPPETS.contains(&"gonzo"));
        assert!(MUPPETS.contains(&"animal"));
        assert_eq!(MUPPETS.len(), 8);
    }
}

/// Generator functions for convenient access to corpus collections
pub mod gen {
    use super::*;
    
    /// Generate a random Muppet character name
    pub fn muppet() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::MUPPETS.len() as u64).0 as usize;
            Tree::singleton(super::MUPPETS[idx])
        })
    }
    
    /// Generate a random animal name
    pub fn animal() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::ANIMALS.len() as u64).0 as usize;
            Tree::singleton(super::ANIMALS[idx])
        })
    }
    
    /// Generate a random color name
    pub fn colour() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::COLOURS.len() as u64).0 as usize;
            Tree::singleton(super::COLOURS[idx])
        })
    }
    
    /// Generate a random fruit name
    pub fn fruit() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::FRUITS.len() as u64).0 as usize;
            Tree::singleton(super::FRUITS[idx])
        })
    }
    
    /// Generate a random vegetable name
    pub fn vegetable() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::VEGETABLES.len() as u64).0 as usize;
            Tree::singleton(super::VEGETABLES[idx])
        })
    }
    
    /// Generate a random weather condition
    pub fn weather() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::WEATHER.len() as u64).0 as usize;
            Tree::singleton(super::WEATHER[idx])
        })
    }
    
    /// Generate a random body of water name
    pub fn water() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::WATERS.len() as u64).0 as usize;
            Tree::singleton(super::WATERS[idx])
        })
    }
    
    /// Generate a random cooking term
    pub fn cooking() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::COOKING.len() as u64).0 as usize;
            Tree::singleton(super::COOKING[idx])
        })
    }
    
    /// Generate a random metasyntactic variable name
    pub fn metasyntactic() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::METASYNTACTIC.len() as u64).0 as usize;
            Tree::singleton(super::METASYNTACTIC[idx])
        })
    }
    
    /// Generate a random "I can eat glass" phrase in various languages and scripts.
    /// Excellent for testing unicode handling, text processing, and internationalization.
    pub fn glass() -> Gen<&'static str> {
        Gen::new(|_size, seed| {
            let idx = seed.next_bounded(super::GLASS.len() as u64).0 as usize;
            Tree::singleton(super::GLASS[idx])
        })
    }
}