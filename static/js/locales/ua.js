/*!
 * FileInput Russian Translations
 *
 * This file must be loaded after 'fileinput.js'. Patterns in braces '{}', or
 * any HTML markup tags in the messages must not be converted or translated.
 *
 * @see http://github.com/kartik-v/bootstrap-fileinput
 * @author CyanoFresh <cyanofresh@gmail.com>
 *
 * NOTE: this file must be saved in UTF-8 encoding.
 
 финин на 60
 */
(function (factory) {
    'use strict';
    if (typeof define === 'function' && define.amd) {
        define(['jquery'], factory);
    } else if (typeof module === 'object' && typeof module.exports === 'object') {
        factory(require('jquery'));
    } else {
        factory(window.jQuery);
    }
}(function ($) {
    "use strict";

    $.fn.fileinputLocales['ua'] = {
        sizeUnits: ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'], 
        bitRateUnits: ['B/s', 'KB/s', 'MB/s', 'GB/s', 'TB/s', 'PB/s', 'EB/s', 'ZB/s', 'YB/s'],
        fileSingle: 'файл',
        filePlural: 'файли',
        browseLabel: 'Вибрати &hellip;',
        removeLabel: 'Видалити',
        removeTitle: 'Очистити вибрані файли',
        cancelLabel: 'Відміна',
        cancelTitle: 'Скасувати поточне завантаження',
        pauseLabel: 'Пауза',
        pauseTitle: 'Призупинити поточне завантаження',
        uploadLabel: 'Завантажити',
        uploadTitle: 'Завантажити вибрані файли',
        msgNo: 'ні',
        msgNoFilesSelected: '',
        msgPaused: 'Припинено',
        msgCancelled: 'Скасовано',
        msgPlaceholder: 'Вибрати {files} ...',
        msgZoomModalHeading: 'Докладне прев\'ю',
        msgFileRequired: 'Необхідно вибрати файл для завантаження.',
        msgSizeTooSmall: 'Файл "{name}" (<b>{size}</b>) має занадто маленький розмір і має бути більшим <b>{minSize}</b>.',
        msgSizeTooLarge: 'Файл "{name}" (<b>{size}</b>) перевищує максимальний розмір <b>{maxSize}</b>.',
        msgFilesTooLess: 'Ви повинні вибрати як мінімум <b>{n}</b> {files} для загрузки.',
        msgFilesTooMany: 'Кількість вибраних файлів <b>({n})</b> перевищує максимально допустиму кількість <b>{m}</b>.',
        msgTotalFilesTooMany: 'Ви можете завантажити максимум <b>{m}</b> файлів (<b>{n}</b> файлів).',
        msgFileNotFound: 'Файл "{name}" не знайдено!',
        msgFileSecured: 'Обмеження безпеки забороняють читати файл "{name}".',
        msgFileNotReadable: 'Файл "{name}" неможливо прочитати.',
        msgFilePreviewAborted: 'Попередній перегляд скасовано для файлу "{name}".',
        msgFilePreviewError: 'Зроблено помилку під час читання файлу "{name}".',
        msgInvalidFileName: 'Невірні або непідтримувані символи у назві файлу "{name}".',
        msgInvalidFileType: 'Заборонений тип файлу "{name}". Тільки "{types}" дозволено.',
        msgInvalidFileExtension: 'Заборонене розширення файлу "{name}". Тільки "{extensions}" дозволені.',
        msgFileTypes: {
            'image': 'image',
            'html': 'HTML',
            'text': 'text',
            'video': 'video',
            'audio': 'audio',
            'flash': 'flash',
            'pdf': 'PDF',
            'object': 'object'
        },
        msgUploadAborted: 'Выгрузка файла прервана',
        msgUploadThreshold: 'Обработка &hellip;',
        msgUploadBegin: 'Инициализация &hellip;',
        msgUploadEnd: 'Готово',
        msgUploadResume: 'Возобновление загрузки &hellip;',
        msgUploadEmpty: 'Недопустимые данные для загрузки',
        msgUploadError: 'Ошибка загрузки',
        msgDeleteError: 'Ошибка удаления',
        msgProgressError: 'Ошибка загрузки',
        msgValidationError: 'Ошибка проверки',
        msgLoading: 'Загрузка файла {index} из {files} &hellip;',
        msgProgress: 'Загрузка файла {index} из {files} - {name} - {percent}% завершено.',
        msgSelected: 'Выбрано файлов: {n}',
        msgProcessing: 'Processing ...',
        msgFoldersNotAllowed: 'Разрешено перетаскивание только файлов! Пропущено {n} папок.',
        msgImageWidthSmall: 'Ширина изображения {name} должна быть не меньше <b>{size} px</b> (detected <b>{dimension} px</b>).',
        msgImageHeightSmall: 'Высота изображения {name} должна быть не меньше <b>{size} px</b> (detected <b>{dimension} px</b>).',
        msgImageWidthLarge: 'Ширина изображения "{name}" не может превышать <b>{size} px</b> (detected <b>{dimension} px</b>).',
        msgImageHeightLarge: 'Высота изображения "{name}" не может превышать <b>{size} px</b> (detected <b>{dimension} px</b>).',
        msgImageResizeError: 'Не удалось получить размеры изображения, чтобы изменить размер.',
        msgImageResizeException: 'Ошибка при изменении размера изображения.<pre>{errors}</pre>',
        msgAjaxError: 'Произошла ошибка при выполнении операции {operation}. Повторите попытку позже!',
        msgAjaxProgressError: 'Не удалось выполнить {operation}',
        msgDuplicateFile: 'Файл "{name}" с размером "{size}" уже был выбран ранее. Пропуск повторяющегося выбора.',
        msgResumableUploadRetriesExceeded: 'Загрузка прервана после <b>{max}</b> попыток для файла <b>{file}</b>! Информация об ошибке: <pre>{error}</pre>',
        msgPendingTime: '{time} осталось',
        msgCalculatingTime: 'расчет оставшегося времени',
        ajaxOperations: {
            deleteThumb: 'удалить файл',
            uploadThumb: 'загрузить файл',
            uploadBatch: 'загрузить пакет файлов',
            uploadExtra: 'загрузка данных с формы'
        },
        dropZoneTitle: 'Перетащите файлы сюда &hellip;',
        dropZoneClickTitle: '<br>(Или щёлкните, чтобы выбрать {files})',
        fileActionSettings: {
            removeTitle: 'Удалить файл',
            uploadTitle: 'Загрузить файл',
            uploadRetryTitle: 'Повторить загрузку',
            downloadTitle: 'Загрузить файл',
            rotateTitle: 'Rotate 90 deg. clockwise',
            zoomTitle: 'Посмотреть детали',
            dragTitle: 'Переместить / Изменить порядок',
            indicatorNewTitle: 'Еще не загружен',
            indicatorSuccessTitle: 'Загружен',
            indicatorErrorTitle: 'Ошибка загрузки',
            indicatorPausedTitle: 'Upload Paused',
            indicatorLoadingTitle:  'Загрузка &hellip;'
        },
        previewZoomButtonTitles: {
            prev: 'Посмотреть предыдущий файл',
            next: 'Посмотреть следующий файл',
            rotate: 'Rotate 90 deg. clockwise',
            toggleheader: 'Переключить заголовок',
            fullscreen: 'Переключить полноэкранный режим',
            borderless: 'Переключить режим без полей',
            close: 'Закрыть подробный предпросмотр'
        }
    };
}));
