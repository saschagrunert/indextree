mod regular_usage;

use indextree::Arena;
use indextree_macros::tree;
use regular_usage::compare_nodes;

#[test]
fn outragous_nesting() {
    let mut arena = Arena::new();

    let root_macro = tree!(&mut arena, "macro root node");
    tree!(
        &mut arena,
        root_macro => {
            "1"=>{"2"=>{"3"=>{"4"=>{"5"=>{"6"=>{"7"=>{"8"=>{"9"=>{"10"=>{"11"=>{"12"=>{"13"=>{"14"=>{"15"=>{"16"=>{"17"=>{"18"=>{"19"=>{"20"=>{"21"=>{"22"=>{"23"=>{"24"=>{"25"=>{"26"=>{"27"=>{"28"=>{"29"=>{"30"=>{"31"=>{"32"=>{"33"=>{"34"=>{"35"=>{"36"=>{"37"=>{"38"=>{"39"=>{"40"=>{"41"=>{"42"=>{"43"=>{"44"=>{"45"=>{"46"=>{"47"=>{"48"=>{"49"=>{"50"=>{"51"=>{"52"=>{"53"=>{"54"=>{"55"=>{"56"=>{"57"=>{"58"=>{"59"=>{"60"=>{"61"=>{"62"=>{"63"=>{"64"=>{"65"=>{"66"=>{"67"=>{"68"=>{"69"=>{"70"=>{"71"=>{"72"=>{"73"=>{"74"=>{"75"=>{"76"=>{"77"=>{"78"=>{"79"=>{"80"=>{"81"=>{"82"=>{"83"=>{"84"=>{"85"=>{"86"=>{"87"=>{"88"=>{"89"=>{"90"=>{"91"=>{"92"=>{"93"=>{"94"=>{"95"=>{"96"=>{"97"=>{"98"=>{"99"=>{"100"}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}
        }
    );

    let root_proc = arena.new_node("proc root node");
    let mut deepest_node = root_proc.append_value("1", &mut arena);
    let owned_strings: Vec<String> = (2..=100).map(|x| x.to_string()).collect();
    for i in &owned_strings {
        deepest_node = deepest_node.append_value(i.as_str(), &mut arena);
    }

    compare_nodes(&arena, root_macro, root_proc);
}

#[test]
fn very_long() {
    let mut arena = Arena::new();

    let root_macro = tree!(
        &mut arena,
        "macro root node" => {
            "1",
            "2",
            "3",
            "4",
            "5",
            "6",
            "7",
            "8",
            "9",
            "10",
            "11",
            "12",
            "13",
            "14",
            "15",
            "16",
            "17",
            "18",
            "19",
            "20",
            "21",
            "22",
            "23",
            "24",
            "25",
            "26",
            "27",
            "28",
            "29",
            "30",
            "31",
            "32",
            "33",
            "34",
            "35",
            "36",
            "37",
            "38",
            "39",
            "40",
            "41",
            "42",
            "43",
            "44",
            "45",
            "46",
            "47",
            "48",
            "49",
            "50",
            "51",
            "52",
            "53",
            "54",
            "55",
            "56",
            "57",
            "58",
            "59",
            "60",
            "61",
            "62",
            "63",
            "64",
            "65",
            "66",
            "67",
            "68",
            "69",
            "70",
            "71",
            "72",
            "73",
            "74",
            "75",
            "76",
            "77",
            "78",
            "79",
            "80",
            "81",
            "82",
            "83",
            "84",
            "85",
            "86",
            "87",
            "88",
            "89",
            "90",
            "91",
            "92",
            "93",
            "94",
            "95",
            "96",
            "97",
            "98",
            "99",
            "100",
        }
    );

    let root_proc = arena.new_node("proc root node");
    let owned_strings: Vec<String> = (1..=100).map(|x| x.to_string()).collect();
    for i in &owned_strings {
        root_proc.append_value(i.as_str(), &mut arena);
    }

    compare_nodes(&arena, root_macro, root_proc);
}
