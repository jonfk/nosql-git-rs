local filepathPathParam = {
  name: 'filepath',
  'in': 'path',
  required: true,
  schema: {
    type: 'string',
  },
};

local successResponse(schema) =
  {
    description: 'Success',
    content: {
      'application/json': {
        schema: {
          '$ref': schema,
        },
      },
    },
  };

local commonRequestParameters = {
  overwrite: {
    type: 'boolean',
  },
  commit_msg: {
    type: 'string',
  },
};

{
  openapi: '3.0.0',
  info: {
    title: 'nosql-git HTTP API',
    description: '',
    version: '0.1.0',
  },
  servers: [
    {
      url: 'http://localhost:8081',
      description: 'Development Server',
    },
  ],
  paths: {
    '/commits/{commit_id}/{filepath}': {
      parameters: [
        {
          name: 'commit_id',
          'in': 'path',
          required: true,
          schema: {
            type: 'string',
          },
        },
        filepathPathParam,
      ],
      get: {
        summary: 'Read file',
        description: 'Read file at commit id and path.',
        operationId: 'get_data',
        responses: {
          '200': {
            '$ref': '#/components/responses/SuccessGetResponse',
          },
        },
      },
      post: {
        summary: 'Create or Update file',
        description: 'Creates or updates file from the version at commit id and path.',
        operationId: 'put_data',
        requestBody: {
          '$ref': '#/components/requestBodies/PostRequestBody',
        },
        responses: {
          '200': {
            '$ref': '#components/responses/SuccessWriteResponse',
          },
        },
      },
      delete: {
        summary: 'Delete file',
        description: 'Delete file from the version at commit id and path.',
        operationId: 'delete',
        requestBody: {
          '$ref': '#/components/requestBodies/DeleteRequestBody',
        },
        responses: {
          '200': {
            '$ref': '#/components/responses/SuccessWriteResponse',
          },
        },
      },
    },
    '/latest/{filepath}': {
      parameters: [
        filepathPathParam,
      ],
      get: {
        summary: 'Read latest file',
        description: 'Read latest version of file at path',
        operation: 'get_latest_data',
        responses: {
          '200': {

            '$ref': '#/components/responses/SuccessGetResponse',
          },
        },
      },
      post: {
        summary: 'Create or Update latest file',
        description: 'Creates or updates latest version of file at path.',
        operationId: 'put_data_latest',
        requestBody: {
          '$ref': '#/components/requestBodies/PostRequestBody',
        },
        responses: {
          '200': {
            '$ref': '#/components/responses/SuccessWriteResponse',
          },
        },
      },
    },
  },

  components: {
    schemas: {
      GitEntry: {
        type: 'object',
        properties: {
          commit_id: {
            type: 'string',
          },
          data: {
            '$ref': '#/components/schemas/GitData',
          },
        },
      },
      GitData: {
        oneOf: [
          {
            '$ref': '#/components/schemas/Dir',
          },
          {
            '$ref': '#/components/schemas/File',
          },
        ],
      },
      Dir: {
        type: 'object',
        properties: {
          entries: {
            type: 'array',
            items: {
              type: 'string',
            },
          },
        },
      },
      File: {
        type: 'object',
        properties: {
          data: {
            type: 'string',
          },
        },
      },

      PutDataReq: {
        type: 'object',
        properties: {
          data: {
            type: 'string',
          },
        } + commonRequestParameters,
        required: ['data'],
      },
      PutDataResp: {
        type: 'object',
        properties: {
          commit_id: {
            type: 'string',
          },
        },
      },
      DeleteReq: {
        type: 'object',
        properties: commonRequestParameters,
      },
    },
    requestBodies: {
      PostRequestBody: {
        description: 'A create or update request',
        required: true,
        content: {
          'application/json': {
            schema: {
              '$ref': '#/components/schemas/PutDataReq',
            },
          },
        },
      },
      DeleteRequestBody: {
        description: 'A delete request',
        required: false,
        content: {
          'application/json': {
            schema: {
              '$ref': '#/components/schemas/DeleteReq',
            },
          },
        },
      },
    },
    responses: {
      SuccessGetResponse:
        {
          description: 'Success',
          content: {
            'application/json': {
              schema: {
                '$ref': '#/components/schemas/GitEntry',
              },
            },
          },
        },
      SuccessWriteResponse: {
        description: 'Success',
        content: {
          'application/json': {
            schema: {
              '$ref': '#/components/schemas/PutDataResp',
            },
          },
        },
      },
    },
  },
}
