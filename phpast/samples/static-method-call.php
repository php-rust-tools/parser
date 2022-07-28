<?php

Schema::table($table, function (Blueprint $table) use ($columns) {
    $table->dropColumn($columns);
});