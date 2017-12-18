# -*- coding: utf-8 -*-

# Form implementation generated from reading ui file '/home/haozeke/.config/calibre/plugins/annotations_resources/dialogs/new_destination.ui'
#
# Created by: PyQt5 UI code generator 5.4.1
#
# WARNING! All changes made in this file will be lost!

from PyQt5 import QtCore, QtGui, QtWidgets

class Ui_Dialog(object):
    def setupUi(self, Dialog):
        Dialog.setObjectName("Dialog")
        Dialog.resize(463, 279)
        sizePolicy = QtWidgets.QSizePolicy(QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Minimum)
        sizePolicy.setHorizontalStretch(0)
        sizePolicy.setVerticalStretch(0)
        sizePolicy.setHeightForWidth(Dialog.sizePolicy().hasHeightForWidth())
        Dialog.setSizePolicy(sizePolicy)
        icon = QtGui.QIcon()
        icon.addPixmap(QtGui.QPixmap(I("lt.png")), QtGui.QIcon.Normal, QtGui.QIcon.Off)
        Dialog.setWindowIcon(icon)
        self.gridLayout_2 = QtWidgets.QGridLayout(Dialog)
        self.gridLayout_2.setObjectName("gridLayout_2")
        self.gl = QtWidgets.QGridLayout()
        self.gl.setObjectName("gl")
        self.header = QtWidgets.QLabel(Dialog)
        font = QtGui.QFont()
        font.setFamily("Lucida Grande")
        font.setPointSize(16)
        self.header.setFont(font)
        self.header.setAlignment(QtCore.Qt.AlignCenter)
        self.header.setObjectName("header")
        self.gl.addWidget(self.header, 0, 0, 1, 2)
        self.move_button = QtWidgets.QPushButton(Dialog)
        self.move_button.setObjectName("move_button")
        self.gl.addWidget(self.move_button, 3, 0, 1, 1)
        self.change_button = QtWidgets.QPushButton(Dialog)
        self.change_button.setObjectName("change_button")
        self.gl.addWidget(self.change_button, 5, 0, 1, 1)
        self.line_2 = QtWidgets.QFrame(Dialog)
        self.line_2.setFrameShape(QtWidgets.QFrame.HLine)
        self.line_2.setFrameShadow(QtWidgets.QFrame.Sunken)
        self.line_2.setObjectName("line_2")
        self.gl.addWidget(self.line_2, 4, 1, 1, 1)
        self.move_label = QtWidgets.QLabel(Dialog)
        sizePolicy = QtWidgets.QSizePolicy(QtWidgets.QSizePolicy.MinimumExpanding, QtWidgets.QSizePolicy.MinimumExpanding)
        sizePolicy.setHorizontalStretch(0)
        sizePolicy.setVerticalStretch(0)
        sizePolicy.setHeightForWidth(self.move_label.sizePolicy().hasHeightForWidth())
        self.move_label.setSizePolicy(sizePolicy)
        self.move_label.setObjectName("move_label")
        self.gl.addWidget(self.move_label, 3, 1, 1, 1)
        self.change_label = QtWidgets.QLabel(Dialog)
        sizePolicy = QtWidgets.QSizePolicy(QtWidgets.QSizePolicy.MinimumExpanding, QtWidgets.QSizePolicy.MinimumExpanding)
        sizePolicy.setHorizontalStretch(0)
        sizePolicy.setVerticalStretch(0)
        sizePolicy.setHeightForWidth(self.change_label.sizePolicy().hasHeightForWidth())
        self.change_label.setSizePolicy(sizePolicy)
        self.change_label.setObjectName("change_label")
        self.gl.addWidget(self.change_label, 5, 1, 1, 1)
        self.line = QtWidgets.QFrame(Dialog)
        self.line.setFrameShape(QtWidgets.QFrame.HLine)
        self.line.setFrameShadow(QtWidgets.QFrame.Sunken)
        self.line.setObjectName("line")
        self.gl.addWidget(self.line, 1, 0, 1, 2)
        spacerItem = QtWidgets.QSpacerItem(20, 8, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.Fixed)
        self.gl.addItem(spacerItem, 2, 1, 1, 1)
        spacerItem1 = QtWidgets.QSpacerItem(20, 8, QtWidgets.QSizePolicy.Minimum, QtWidgets.QSizePolicy.MinimumExpanding)
        self.gl.addItem(spacerItem1, 6, 1, 1, 1)
        self.gridLayout_2.addLayout(self.gl, 0, 0, 1, 1)
        self.bb = QtWidgets.QDialogButtonBox(Dialog)
        self.bb.setStandardButtons(QtWidgets.QDialogButtonBox.Cancel)
        self.bb.setObjectName("bb")
        self.gridLayout_2.addWidget(self.bb, 1, 0, 1, 1)

        self.retranslateUi(Dialog)
        QtCore.QMetaObject.connectSlotsByName(Dialog)

    def retranslateUi(self, Dialog):
        _translate = QtCore.QCoreApplication.translate
        Dialog.setWindowTitle(_translate("Dialog", "Annotations destination"))
        self.header.setText(_translate("Dialog", "Move annotations or change destination?"))
        self.move_button.setText(_translate("Dialog", "Move"))
        self.change_button.setText(_translate("Dialog", "Change"))
        self.move_label.setText(_translate("Dialog", "<html><head/><body><p>&bull; Move existing annotations from <span style=\" font-weight:600;\">{old}</span> to <span style=\" font-weight:600;\">{new}</span>.<br/>&bull; Existing annotations will be removed from <span style=\" font-weight:600;\">{old}</span>.<br/>&bull; Newly imported annotations will be added to <span style=\" font-weight:600;\">{new}</span>.</p></body></html>"))
        self.change_label.setText(_translate("Dialog", "<html><head/><body><p>&bull; Change annotations storage from <span style=\" font-weight:600;\">{old}</span> to <span style=\" font-weight:600;\">{new}</span>.<br/>&bull; Existing annotations will remain in <span style=\" font-weight:600;\">{old}</span>.<br/>&bull; Newly imported annotations will be added to <span style=\" font-weight:600;\">{new}</span>.</p></body></html>"))

