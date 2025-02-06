<?php

// Print the length of $_SERVER["TEST_PROP"]
echo strlen($_SERVER["TEST_PROP"]);
echo $_SERVER["TEST_PROP"];
echo " <- Server TEST_PROP\n";

// Get the "TEST" cookie and print its value
if (isset($_COOKIE['TEST'])) {
    echo "Cookie 'TEST' is set!\n";
    echo "Value is: " . $_COOKIE['TEST'];
} else {
    echo "Cookie 'TEST' is not set!\n";
}

// Get the "TEST" environment variable and print its value
if (getenv('TEST')) {
    echo "Environment variable 'TEST' is set!\n";
    echo "Value is: " . getenv('TEST');
    echo "\n";
} else {
    echo "Environment variable 'TEST' is not set!\n";
}

echo "Hello, world!\n";