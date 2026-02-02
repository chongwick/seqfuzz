<?php
require 'vendor/autoload.php';

use PhpParser\Error; use PhpParser\NodeDumper;
use PhpParser\ParserFactory;
use PhpParser\PrettyPrinter;

$parser = (new ParserFactory())->createForNewestSupportedVersion();//->create(ParserFactory::PREFER_PHP5);
$prettyPrinter = new PrettyPrinter\Standard;
$nodeDumper = new PhpParser\NodeDumper;
#$targetFile = "./sample2.php";
$targetFile = $argv[1];
$lines = file($targetFile);
try{
    $file_contents = file_get_contents($targetFile);
    $stmts = $parser->parse($file_contents);
    #echo $stmts;
} catch (Error $e) {
    #echo "not good parse";
}
echo json_encode($stmts, JSON_PRETTY_PRINT);
