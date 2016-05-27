# chrome-pak-customizer    [![Download](https://img.shields.io/github/downloads/myfreeer/chrome-pak-customizer/total.svg)](https://github.com/myfreeer/chrome-pak-customizer/releases)    [![Latest Release](https://img.shields.io/github/release/myfreeer/chrome-pak-customizer.svg)](https://github.com/myfreeer/chrome-pak-customizer/releases/latest)    [![GitHub license](https://img.shields.io/github/license/myfreeer/chrome-pak-customizer.svg)](LICENSE)     
a simple batch tool to customize pak files in chrome or chromium-based browser  

forked from https://bitbucket.org/hikipro/node-chrome-pak/src

#Usage
##step1
###[download a full package of my tool](https://github.com/myfreeer/chrome-pak-customizer/releases/latest) and put it into a folder
###put chrome_100_percent.pak into the folder above
###run unpack.cmd
##step2
###check the folder 'unpacked' for files to modify and move the modified file to the folder 'modified'
###run replace.cmd
###done
##settings.ini
###the file settings.ini is created for advanced users, make sure you understand everything provided before using it
###it can work fine without settings.ini
##[Leanify](https://github.com/JayXon/Leanify)
###Leanify is a open-source tool created by [JayXon](https://github.com/JayXon) to reduce the size of png files (seems lossless)
###get leanify [HERE](https://github.com/JayXon/Leanify/releases) and put Leanify.exe together with replace.cmd and it will be automatically used
#Blink(Chromium)内核浏览器皮肤修改工具
#使用说明：
###1.[下载完整包](https://github.com/myfreeer/chrome-pak-customizer/releases/latest)并解压到同一文件夹内
###2.找到 chrome_100_percent.pak 复制到上面那个文件夹内
###3.运行unpack.cmd
###4.在unpacked文件夹内找到想修改的文件，修改后放到modified（不要改文件名）
###5.运行replace.cmd
###6.完成
