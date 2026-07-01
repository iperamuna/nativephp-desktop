<?php

namespace Native\Desktop\Drivers\Tauri\Commands;

use Illuminate\Console\Command;
use Native\Desktop\Drivers\Tauri\TauriServiceProvider;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Filesystem\Filesystem;

use function Laravel\Prompts\intro;

#[AsCommand(
    name: 'native:reset',
    description: 'Clear all build and dist files for NativePHP Tauri app',
)]
class ResetCommand extends Command
{
    protected $signature = 'native:reset {--with-app-data : Clear the app data as well}';

    public function handle(): int
    {
        intro('Clearing build and dist directories...');

        $filesystem = new Filesystem;

        // Removing and recreating the native serve resource path
        $nativeServeResourcePath = realpath(TauriServiceProvider::buildPath('app'));
        if ($filesystem->exists($nativeServeResourcePath)) {
            $this->line('Clearing: '.$nativeServeResourcePath);
            $filesystem->remove($nativeServeResourcePath);
            $filesystem->mkdir($nativeServeResourcePath);
        }

        // Removing the bundling directories
        $bundlingPath = base_path('build/');
        $this->line('Clearing: '.$bundlingPath);

        if ($filesystem->exists($bundlingPath)) {
            $filesystem->remove($bundlingPath);
        }

        // Removing the built path
        $builtPath = base_path('dist/');
        $this->line('Clearing: '.$builtPath);

        if ($filesystem->exists($builtPath)) {
            $filesystem->remove($builtPath);
        }

        if ($this->option('with-app-data')) {
            foreach ([true, false] as $developmentMode) {
                $appName = $this->setAppNameAndVersion($developmentMode);

                if (! empty($appName)) {
                    $appDataPath = $this->appDataDirectory($appName);
                    $this->line('Clearing: '.$appDataPath);

                    if ($filesystem->exists($appDataPath)) {
                        $filesystem->remove($appDataPath);
                    }
                }
            }
        }

        return 0;
    }

    protected function setAppNameAndVersion(bool $developmentMode = false): string
    {
        $appName = config('app.name');
        return $developmentMode ? $appName.' - Development' : $appName;
    }

    protected function appDataDirectory(string $name): string
    {
        return match (PHP_OS_FAMILY) {
            'Darwin' => $_SERVER['HOME'].'/Library/Application Support/'.$name,
            'Linux' => $_SERVER['XDG_CONFIG_HOME'] ?? $_SERVER['HOME'].'/.config/'.$name,
            'Windows' => $_SERVER['APPDATA'].'/'.$name,
            default => $_SERVER['HOME'].'/.config/'.$name,
        };
    }
}
