import React, { useState } from 'react';
import { InboxOutlined } from '@ant-design/icons';
import type { UploadFile, UploadProps } from 'antd';
import { Button, message, Upload } from 'antd';
import axios from 'axios';
import { NotificationInstance } from 'antd/es/notification/interface';

const { Dragger } = Upload;

// const UploadImage: React.FC<{useremail: string}> = (props) => {
const UploadImage: React.FC<{ messageClient: NotificationInstance, fileList: UploadFile[], setFileList: Function }> = (props) => {
  const { messageClient, fileList, setFileList } = props;

  let properties: UploadProps = {
    name: 'file',
    multiple: true,
    fileList: fileList,
    beforeUpload(file, FileList) {
      if (sessionStorage.getItem('useremail')) {
        return true;
      } else {
        messageClient.error({
          message: `Forbidden Operation!`,
          description: "You should sign in first!",
          placement: 'topLeft',
          duration: 2,
        });
        return false;
      }
    },
    customRequest(options) {
      const { onSuccess, onError, file, filename } = options;
      axios.post(`/${sessionStorage.getItem('useremail')}/upload_pic`, {
        file: file
      }, {
        headers:{
          'Content-Type': "multipart/form-data"
        }
      }).then(res => {
        onSuccess!(file);
      })
      .catch(err=>{
        const error = new Error(err);
        onError!(error);
      });
    },
    onChange(info) {
      const { status } = info.file;
      if (status === 'done') {
        if (info.file.response.code === 200) {
          messageClient.success({
            message: `Uploading Succeeded`,
            description: `The image named ${info.file.name} has been uploaded.`,
            placement: 'topLeft',
            duration: 2,
          });
        }
      } else if (status === 'error') {
        messageClient.error({
          message: `Upload Failed`,
          description: `The image named ${info.file.name} was failed to be uploaded. Please try it again later!`,
          placement: 'topLeft',
          duration: 2,
        });
      }
      // console.log(info.fileList);
      setFileList([...info.fileList]);
    },
    onDrop(e) {
      console.log('Catched dropped files', e.dataTransfer.files);
    },
    onRemove(file) {
      let file_list = fileList.filter((item, index, array)=>{
        return item.name != file.name;
      })
      setFileList([...file_list]);
    },
  };

  return (
    <>
      <Dragger {...properties}>
          <p className="ant-upload-drag-icon">
              <InboxOutlined />
          </p>
          <p className="ant-upload-text">Click or drag file to this area to upload</p>
          <p className="ant-upload-hint">
              Support for a single or bulk upload. Strictly prohibited from uploading company data or other
              banned files.
          </p>
      </Dragger>
    </>
  );
};

export default UploadImage;