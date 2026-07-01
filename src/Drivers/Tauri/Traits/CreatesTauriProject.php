<?php

namespace Native\Desktop\Drivers\Tauri\Traits;

use Illuminate\Support\Facades\File;
use Native\Desktop\Support\Composer;
use Symfony\Component\Filesystem\Filesystem;

trait CreatesTauriProject
{
    public function createTauriProject($installPath)
    {
        $sourcePath = Composer::desktopPackagePath('resources/tauri');

        File::ensureDirectoryExists($installPath, 0755, true);

        (new Filesystem)->mirror(
            $sourcePath,
            $installPath,
            options: [
                'override' => true,
                'delete' => true,
            ]
        );
    }
}
