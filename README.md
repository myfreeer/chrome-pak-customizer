# chrome-pak-customizer
a simple batch tool to customize pak files in chrome or chromium-based browser  

forked from https://bitbucket.org/hikipro/node-chrome-pak/src

#Usage
##step1
###download my tool and put it into a folder
###download [node.exe](https://nodejs.org/download/release/latest/) and put it into the folder above
###put chrome_100_percent.pak into the folder above
###run unpack.cmd
##step2
###check the folder unpacked for files to modify and move the modified file to the folder modified
###run replace.cmd
###done
##settings.ini
###the file settings.ini is created for advanced users, make sure you understand everything provided before using it
###it can work fine without settings.ini
##[Leanify](https://github.com/JayXon/Leanify)
###Leanify is a opensource tool to reduce the size of png files (seems lossless)
###get leanify [HERE](https://github.com/JayXon/Leanify/releases) and put Leanify.exe together with replace.cmd and it will be automatically used
#使用说明：
###1.根据系统版本下载完整包解压到同一文件夹内
###2.找到 chrome_100_percent.pak 复制到上面那个文件夹内
###3.运行unpack.cmd
###4.在unpacked文件夹内找到想修改的文件，修改后放到modified（不要改文件名）
###5.运行replace.cmd
###6.完成
