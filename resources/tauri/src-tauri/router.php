<?php
$appPath = getenv('APP_PATH');
if ($appPath) {
    chdir($appPath . '/public');
}
return require_once $appPath . '/vendor/laravel/framework/src/Illuminate/Foundation/resources/server.php';
