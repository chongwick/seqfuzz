<?php
#$data = [];
#
#foreach ($data as $item) {
#    echo "Will not run, array is empty.";
#}

#$array = new ArrayIterator(["x", "y", "z"]);
#
#foreach ($array as $item) {
#    echo $item . "\n";
#}

#$numbers = [1, 2, 3];
#
#foreach ($numbers as &$n) {
#        #$h=$n + 4;
#    $n *= 2;  // modify in-place
#}
#unset($n); // important to break the reference
#
print_r($numbers); // [2, 4, 6]

$pairs = [[1, "one"], [2, "two"], [3, "three"]];

foreach ($pairs as list($num, $word)) {
    echo "$num = $word\n";
}
#
#class User {
#    public $name = "Alice";
#    public $email = "alice@example.com";
#}
#
#$user = new User();
#
#foreach ($user as $property => $value) {
#    echo "$property => $value\n";
#}
#
#$fruits = ["a" => "apple", "b" => "banana", "c" => "cherry"];
#
#foreach ($fruits as $key => $value) {
#    echo "Key: $key, Value: $value\n";
#}
#
#$fruits = ["apple", "banana", "cherry"];
#
#foreach ($fruits as $fruit) {
#    echo $fruit . "\n";
#}
#
