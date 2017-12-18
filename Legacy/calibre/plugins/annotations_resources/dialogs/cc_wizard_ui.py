# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file '/home/haozeke/.config/calibre/plugins/annotations_resources/dialogs/cc_wizard.ui'
#
# Created by: PyQt5 UI code generator 5.4.1
#
# WARNING! All changes made in this file will be lost!

from PyQt5 import QtCore, QtGui, QtWidgets

class Ui_Dialog(object):
    def setupUi(self, Dialog):
        Dialog.setObjectName("Dialog")
        Dialog.resize(400, 175)
        icon = QtGui.QIcon()
        icon.addPixmap(QtGui.QPixmap(I("lt.png")), QtGui.QIcon.Normal, QtGui.QIcon.Off)
        Dialog.setWindowIcon(icon)
        self.gridLayout_2 = QtWidgets.QGridLayout(Dialog)
        self.gridLayout_2.setObjectName("gridLayout_2")
        self.gl = QtWidgets.QGridLayout()
        self.gl.setObjectName("gl")
        self.header = QtWidgets.QLabel(Dialog)
        font = QtGui.QFont()
        font.setPointSize(16)
        self.header.setFont(font)
        self.header.setAlignment(QtCore.Qt.AlignCenter)
        self.header.setObjectName("header")
        self.gl.addWidget(self.header, 0, 1, 1, 1)
        self.calibre_destination_le = QtWidgets.QLineEdit(Dialog)
        self.calibre_destination_le.setObjectName("calibre_destination_le")
        self.gl.addWidget(self.calibre_destination_le, 3, 1, 1, 1)
        self.icon = QtWidgets.QLabel(Dialog)
        self.icon.setAlignment(QtCore.Qt.AlignCenter)
        self.icon.setObjectName("icon")
        self.gl.addWidget(self.icon, 0, 0, 1, 1)
        self.step_1 = QtWidgets.QLabel(Dialog)
        self.step_1.setWordWrap(True)
        self.step_1.setObjectName("step_1")
        self.gl.addWidget(self.step_1, 2, 1, 1, 1)
        spacerItem = QtWidgets.QSpacerItem(40, 20, QtWidgets.QSizePolicy.Fixed, QtWidgets.QSizePolicy.Minimum)
        self.gl.addItem(spacerItem, 3, 2, 1, 1)
        spacerItem1 = QtWidgets.QSpacerItem(40, 20, QtWidgets.QSizePolicy.Fixed, QtWidgets.QSizePolicy.Minimum)
        self.gl.addItem(spacerItem1, 3, 0, 1, 1)
        self.line = QtWidgets.QFrame(Dialog)
        self.line.setFrameShape(QtWidgets.QFrame.HLine)
        self.line.setFrameShadow(QtWidgets.QFrame.Sunken)
        self.line.setObjectName("line")
        self.gl.addWidget(self.line, 1, 1, 1, 1)
        spacerItem2 = QtWidgets.QSpacerItem(20, 40, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Expanding)
        self.gl.addItem(spacerItem2, 4, 1, 1, 1)
        self.gridLayout_2.addLayout(self.gl, 0, 0, 1, 1)
        self.bb = QtWidgets.QDialogButtonBox(Dialog)
        self.bb.setStandardButtons(QtWidgets.QDialogButtonBox.Cancel)
        self.bb.setObjectName("bb")
        self.gridLayout_2.addWidget(self.bb, 1, 0, 1, 1)

        self.retranslateUi(Dialog)
        QtCore.QMetaObject.connectSlotsByName(Dialog)

    def retranslateUi(self, Dialog):
        _translate = QtCore.QCoreApplication.translate
        Dialog.setWindowTitle(_translate("Dialog", "Custom column wizard"))
        self.header.setText(_translate("Dialog", "Custom column wizard"))
        self.icon.setText(_translate("Dialog", "icon"))
        self.step_1.setText(_translate("Dialog", "1. Step 1"))

