<?php
$q = 3;
$w = 4;
$r = $q - $w;
if($q > 14){
	$r = 2;
}


#$f = fopen('php://temp/maxmemory:1024', 'r+');
#fwrite($f, str_repeat("1", 738));
#fseek($f, 0, SEEK_SET);
#fwrite($f, str_repeat("2", 512));
#
#fseek($f, 0, SEEK_SET);
#var_dump(fread($f, 16));
#
#fseek($f, 0, SEEK_END);
#var_dump(ftell($f));

?>
