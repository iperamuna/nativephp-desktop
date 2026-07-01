<?php

namespace Native\Desktop\Drivers\Tauri;

use Illuminate\Foundation\Application;
use Native\Desktop\Builder\Builder;
use Native\Desktop\Drivers\Tauri\Commands\BuildCommand;
use Native\Desktop\Drivers\Tauri\Commands\InstallCommand;
use Native\Desktop\Drivers\Tauri\Commands\PublishCommand;
use Native\Desktop\Drivers\Tauri\Commands\ResetCommand;
use Native\Desktop\Drivers\Tauri\Commands\RunCommand;
use Native\Desktop\Events\LivewireDispatcher;
use Native\Desktop\Support\Composer;
use Spatie\LaravelPackageTools\Package;
use Spatie\LaravelPackageTools\PackageServiceProvider;

class TauriServiceProvider extends PackageServiceProvider
{
    public static function tauriPath(string $path = '')
    {
        // Will use the published tauri project, or fallback to the vendor default
        $publishedProjectPath = base_path("nativephp/tauri/{$path}");

        return file_exists("{$publishedProjectPath}/package.json") || file_exists("{$publishedProjectPath}/src-tauri/Cargo.toml")
            ? $publishedProjectPath
            : Composer::desktopPackagePath("resources/tauri/{$path}");
    }

    public static function buildPath(string $path = '')
    {
        return Composer::desktopPackagePath("resources/build/{$path}");
    }

    public function configurePackage(Package $package): void
    {
        $package
            ->name('nativephp-tauri')
            ->hasCommands([
                InstallCommand::class,
                RunCommand::class,
                BuildCommand::class,
                PublishCommand::class,
                ResetCommand::class,
            ]);
    }

    public function packageRegistered(): void
    {
        // TODO: Implement Tauri UpdaterManager once Tauri bridging is ready
        // $this->app->bind('nativephp.updater', function (Application $app) {
        //     return new UpdaterManager($app);
        // });

        $this->app->bind(Builder::class, function () {
            return Builder::make(
                buildPath: self::buildPath()
            );
        });
    }

    public function packageBooted(): void
    {
        app(LivewireDispatcher::class)->register();
    }
}
