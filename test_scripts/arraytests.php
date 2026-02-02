<?php
#$arg1 = "second arg";
#$arg2 = "third arg";
#
class sample
{
  public function __toString() {
    return "Object";
  }
}

$file_handle = fopen(__FILE__, 'r');

$values = array(

      // int data
      0,
      1,
      12345,
      -2345,

      // float data
      10.5,
      -10.5,
      10.1234567e10,
      10.7654321E-10,
      .5,
      'banan' => 'a',

      // array data
      array(),
      array(0),
      array(1),
      array(1, 2),
      array('color' => 'red', 'item' => 'pen'),

      // boolean data
      true,
      false,
      TRUE,
      FALSE,

      // empty data
      "",
      '',

      // object data
      new sample(),

      // resource data
      $file_handle
);


#$count = 1;
#foreach($values as $value) {
#  echo "\n-- Iteration $count --\n";
#
#  // with default argument
#  try {
#    var_dump(sprintf($value));
#  } catch (TypeError $exception) {
#    echo $exception->getMessage() . "\n";
#  }
#
#  // with two arguments
#  try {
#    var_dump(sprintf($value, $arg1));
#  } catch (TypeError $exception) {
#    echo $exception->getMessage() . "\n";
#  }
#
#  // with three arguments
#  try {
#    var_dump(sprintf($value, $arg1, $arg2));
#  } catch (TypeError $exception) {
#    echo $exception->getMessage() . "\n";
#  }
#
#  $count++;
#}
#
#// close the resource
#fclose($file_handle);
#
#echo "Done";
?>
