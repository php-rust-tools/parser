<?php

$guess = readline("Guess a number between 1 and 3: ");
$number = rand(1, 3);

if ($guess == $number) {
    echo "You guessed the number correctly, well done!";
} else {
    echo "The correct answer is " . $number . ". Better luck next time!";
}